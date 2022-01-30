use diesel::{Insertable, QueryDsl, RunQueryDsl};
use std::time::SystemTime;

use crate::diesel::ExpressionMethods;
use crate::error::MyError;
use crate::schema::feeds::dsl::feeds as db_feeds;
use crate::schema::{backup_feeds, feeds, subscriptions};
use crate::Pool;

use super::Feed;

#[derive(Debug, Insertable)]
#[table_name = "feeds"]
pub struct NewFeed<'a> {
    pub canvas_id: &'a str,
    pub url: &'a str,
    pub last_update: SystemTime,
}

#[derive(Debug, Queryable)]
pub struct DbFeed {
    pub id: i32,
    pub canvas_id: String,
    pub url: String,
    pub last_update: SystemTime,
}

#[derive(Debug, Insertable)]
#[table_name = "backup_feeds"]
pub struct NewBackupFeed<'a> {
    pub feed_id: i32,
    pub url: &'a str,
}

#[derive(Debug, Queryable)]
pub struct DbBackupFeed {
    pub id: i32,
    pub feed_id: i32,
    pub url: String,
}

#[derive(Debug, Insertable)]
#[table_name = "subscriptions"]
pub struct NewSubsription<'a> {
    pub server_id: &'a str,
    pub channel_id: &'a str,
    pub feed_id: i32,
}

#[derive(Debug, Queryable)]
pub struct Subscription {
    pub id: i32,
    pub server_id: String,
    pub channel_id: String,
    pub feed_id: i32,
}

impl DbFeed {
    pub fn get_by_canvas_id(
        search_canvas_id: &str,
        pool: &Pool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = pool.get()?;

        Ok(db_feeds
            .filter(feeds::canvas_id.eq(search_canvas_id))
            .get_result(&conn)?)
    }

    pub fn get_all(pool: &Pool) -> Result<Vec<DbFeed>, MyError> {
        let conn = pool.get()?;

        Ok(db_feeds.load::<DbFeed>(&conn)?)
    }
}

impl Subscription {
    pub async fn add(
        server_id: &str,
        channel_id: &str,
        url: &str,
        pool: &Pool,
    ) -> Result<(), MyError> {
        // Get feed canvas_id
        let feed = dbg!(Feed::from_url(url).await?);

        let conn = pool.get()?;

        let tmp: Result<i32, _> = db_feeds
            .filter(feeds::canvas_id.eq(&feed.id))
            .select(feeds::id)
            .get_result(&conn);
        let feed_id = if let Ok(id) = tmp {
            id
        } else if Err(diesel::result::Error::NotFound) == tmp {
            // Add feed to db
            Feed::add(url, pool).await?;
            // Get the pk of new entry
            db_feeds
                .filter(feeds::canvas_id.eq(&feed.id))
                .select(feeds::id)
                .get_result(&conn)?
        } else {
            tmp?
        };

        let new_subscription = NewSubsription {
            server_id,
            channel_id,
            feed_id,
        };

        let _ret: usize = diesel::insert_into(subscriptions::table)
            .values(&new_subscription)
            .execute(&conn)?;

        Ok(())
    }
}
