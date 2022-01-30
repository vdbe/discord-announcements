use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use discord_announcements::{Feed, Pool, Subscription};
use dotenv::dotenv;
use proto_canvas_rss::canvas_rss_server::{CanvasRss, CanvasRssServer};
use proto_canvas_rss::{
    AnnouncementReply, FeedReply, HelloReply, HelloRequest, ListFeedsRequest,
    NewAnnouncementsRequest, SubscribeRequest, SubscribeResponse,
};
use std::time::SystemTime;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

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
        let feeds = Feed::get_all(&self.pool).await.unwrap();

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
        let feeds = Feed::get_new(&self.pool).await.unwrap();

        let (tx, rx) = mpsc::channel(4);

        if let Some(feeds) = feeds {
            tokio::spawn(async move {
                for feed in feeds {
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

        // Check if feed url is valid
        //if Feed::from_url(subscribe_request.feed).await.is_err() {
        //    return Ok(Response::new(SubscribeResponse { success: false }));
        //}
        let subscribe_response = match Subscription::add(
            &subscribe_request.guild_id,
            &subscribe_request.channel_id,
            &subscribe_request.feed,
            &self.pool,
        )
        .await
        {
            Ok(_) => SubscribeResponse { success: true },
            Err(_) => SubscribeResponse { success: false },
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
    let greeter = CanvasRssService { pool };

    Server::builder()
        .add_service(CanvasRssServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
