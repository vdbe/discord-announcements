use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use discord_announcements::{DbError, DbSubscription, Feed, Pool};
use dotenv::dotenv;
use std::time::SystemTime;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

use discord_announcements::{FeedError, MyError};
use proto_canvas_rss::canvas_rss_server::{CanvasRss, CanvasRssServer};
use proto_canvas_rss::{
    AnnouncementReply, FeedReply, HelloReply, HelloRequest, ListFeedsRequest,
    NewAnnouncementsRequest, SubscribeRequest, SubscribeResponse, Subscriber,
};

pub mod proto_canvas_rss {
    tonic::include_proto!("canvasrss");
}

pub struct CanvasRssService {
    pool: Pool,
}

#[tonic::async_trait]
impl CanvasRss for CanvasRssService {
    type ListFeedsStream = ReceiverStream<Result<FeedReply, Status>>;
    type NewAnnouncementsStream = ReceiverStream<Result<FeedReply, Status>>;

    async fn list_feeds(
        &self,
        request: tonic::Request<ListFeedsRequest>,
    ) -> Result<tonic::Response<Self::ListFeedsStream>, tonic::Status> {
        let (check_date, after) = match request.into_inner().after {
            Some(after) => match SystemTime::try_from(after) {
                Ok(systemtime) => (true, systemtime),
                Err(_) => (false, SystemTime::UNIX_EPOCH),
            },
            None => (false, SystemTime::UNIX_EPOCH),
        };
        let feeds = if let Ok(feeds) = Feed::get_all(&self.pool).await {
            feeds
        } else {
            return Err(tonic::Status::new(
                tonic::Code::Internal,
                "Failed to retreive feeds",
            ));
        };

        let (tx, rx) = mpsc::channel(4);

        if let Some(feeds) = feeds {
            tokio::spawn(async move {
                //for rss_feed in &rss_feeds[..] {
                for feed in feeds {
                    //let mut feed = Feed::from_url(*rss_feed).await.unwrap();
                    let mut feed = feed;
                    if check_date {
                        feed.after(after);
                        if feed.announcements.is_empty() {
                            continue;
                        }
                    }

                    let mut announcements: Vec<AnnouncementReply> = Vec::new();

                    for announcement in feed.announcements {
                        announcements.push(AnnouncementReply {
                            title: announcement.title,
                            published: Some(announcement.published.into()),
                            link: announcement.link.href,
                            author: announcement.author.name,
                            content: announcement.content.content,
                        });
                    }

                    let feed_reply = FeedReply {
                        id: feed.id,
                        announcements,
                        subscribers: Vec::new(),
                    };

                    tx.send(Ok(feed_reply.clone())).await.unwrap();
                }
            });
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn new_announcements(
        &self,
        _request: tonic::Request<NewAnnouncementsRequest>,
    ) -> Result<tonic::Response<Self::NewAnnouncementsStream>, tonic::Status> {
        let feeds = if let Ok(feeds) = Feed::get_new(&self.pool).await {
            feeds
        } else {
            return Err(tonic::Status::new(
                tonic::Code::Internal,
                "Failed to retreive feeds",
            ));
        };

        let (tx, rx) = mpsc::channel(4);

        if let Some(feeds) = feeds {
            tokio::spawn(async move {
                for (feed, subscribers) in feeds {
                    let mut announcements: Vec<AnnouncementReply> = Vec::new();

                    for announcement in feed.announcements {
                        announcements.push(AnnouncementReply {
                            title: announcement.title,
                            published: Some(announcement.published.into()),
                            link: announcement.link.href,
                            author: announcement.author.name,
                            content: announcement.content.content,
                        });
                    }

                    let subscribers = subscribers
                        .iter()
                        .map(|s| Subscriber {
                            server_id: s.0.clone(),
                            channel_id: s.1.clone(),
                        })
                        .collect();

                    let feed_reply = FeedReply {
                        id: feed.id,
                        announcements,
                        subscribers,
                    };

                    tx.send(Ok(feed_reply.clone())).await.unwrap();
                }
            });
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got hello request");

        //_ = DbFeed::get_by_canvas_id("asdf", &self.pool);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }

    async fn subscribe(
        &self,
        request: tonic::Request<SubscribeRequest>,
    ) -> Result<tonic::Response<SubscribeResponse>, tonic::Status> {
        let subscribe_request = request.into_inner();
        let subscriber = match subscribe_request.subscriber {
            Some(x) => x,
            None => Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "No subscriber provided",
            ))?,
        };

        let subscribe_response = match DbSubscription::add(
            &subscriber.server_id,
            &subscriber.channel_id,
            &subscribe_request.feed,
            &self.pool,
        )
        .await
        {
            Ok(title) => SubscribeResponse {
                success: true,
                message: format!("Placed a subscription for \'{title}\'"),
            },
            Err(MyError::Feed(FeedError::InvalidFeedUrl(_))) => SubscribeResponse {
                success: false,
                message: String::from("Looks like you passed an invalid feed url"),
            },
            Err(MyError::Feed(FeedError::De(_))) => SubscribeResponse {
                success: false,
                message: String::from("Failed to read the url as a announcement feed"),
            },
            Err(MyError::Db(DbError::UniqueViolation)) => SubscribeResponse {
                success: false,
                message: String::from("This channel is already subscribed to that feed"),
            },
            Err(_) => SubscribeResponse {
                success: false,
                message: String::from("Oops something went wrong"),
            },
        };

        Ok(Response::new(subscribe_response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool = r2d2::Pool::builder().build(manager).unwrap();

    let addr = "[::1]:50051".parse()?;
    let canvas_rss = CanvasRssService { pool };

    Server::builder()
        .add_service(CanvasRssServer::new(canvas_rss))
        .serve(addr)
        .await?;

    Ok(())
}
