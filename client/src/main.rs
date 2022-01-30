use proto_canvas_rss::canvas_rss_client::CanvasRssClient;
use proto_canvas_rss::{
    HelloRequest, ListFeedsRequest, NewAnnouncementsRequest, SubscribeRequest, Subscriber,
};
use std::error::Error;
use std::time::SystemTime;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tonic::transport::Channel;
use tonic::Request;

pub mod proto_canvas_rss {
    tonic::include_proto!("canvasrss");
}

#[allow(dead_code)]
async fn get_feeds(client: &mut CanvasRssClient<Channel>) -> Result<(), Box<dyn Error>> {
    let time = "2022-01-01T00:00:00+02:00";
    let date: SystemTime = OffsetDateTime::parse(time, &Rfc3339).unwrap().into();
    let list_feeds_request = ListFeedsRequest {
        after: Some(date.into()),
    };

    let mut stream = client
        .list_feeds(Request::new(list_feeds_request))
        .await?
        .into_inner();

    while let Some(feed) = stream.message().await? {
        dbg!(feed);
    }

    Ok(())
}

#[allow(dead_code)]
async fn get_new_announcements(
    client: &mut CanvasRssClient<Channel>,
) -> Result<(), Box<dyn Error>> {
    let new_announcements_request = NewAnnouncementsRequest {};

    use std::time::Instant;
    let now = Instant::now();
    let mut stream = client
        .new_announcements(Request::new(new_announcements_request))
        .await?
        .into_inner();
    let elapsed = now.elapsed();

    let mut announcement_count = 0;
    while let Some(feed) = stream.message().await? {
        announcement_count += feed.announcements.len();
    }

    println!("{announcement_count} new announcements in {elapsed:.2?}");

    Ok(())
}

#[allow(dead_code)]
async fn subscribe(
    guild_id: String,
    channel_id: String,
    feed: String,
    client: &mut CanvasRssClient<Channel>,
) -> Result<(), Box<dyn Error>> {
    let subscriber = Subscriber {
        server_id: guild_id,
        channel_id: channel_id,
    };
    let subscribe_request = SubscribeRequest {
        feed,
        subscriber: Some(subscriber),
    };

    let response = client
        .subscribe(Request::new(subscribe_request))
        .await?
        .into_inner();

    dbg!(response);

    Ok(())
}

#[allow(dead_code)]
async fn hello(client: &mut CanvasRssClient<Channel>) -> Result<(), Box<dyn Error>> {
    let hello_request = HelloRequest {
        name: "test".into(),
    };

    let response = client
        .say_hello(Request::new(hello_request))
        .await?
        .into_inner();

    dbg!(response);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CanvasRssClient::connect("http://[::1]:50051").await?;

    //get_feeds(&mut client).await?;
    get_new_announcements(&mut client).await?;
    //subscribe(&mut client).await?;
    //hello(&mut client).await?;

    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    let file = File::open("feeds.txt")?;
    let buf = BufReader::new(file);
    let rss_feeds: Vec<String> = buf
        .lines()
        .map(|l| l.expect("Could not parse line"))
        .collect();

    for feed in rss_feeds {
        subscribe(
            "804292740287561729".into(),
            "804306034712641546".into(),
            feed,
            &mut client,
        )
        .await?;
    }
    Ok(())
}
