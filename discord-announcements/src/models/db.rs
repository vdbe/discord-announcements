use diesel::{Insertable, QueryDsl, RunQueryDsl};
use std::time::SystemTime;

use crate::diesel::ExpressionMethods;
use crate::error::{DbError, MyError};
use crate::schema::feeds::dsl::feeds as db_feeds;
use crate::schema::subscriptions::dsl::subscriptions as db_subscriptions;

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
pub struct DbSubscription {
    pub id: i32,
    pub server_id: String,
    pub channel_id: String,
    pub feed_id: i32,
}

impl DbFeed {
    pub fn get_by_canvas_id(search_canvas_id: &str, pool: &Pool) -> Result<Option<Self>, DbError> {
        let conn = pool.get()?;

        match db_feeds
            .filter(feeds::canvas_id.eq(search_canvas_id))
            .get_result(&conn)
        {
            Ok(f) => Ok(Some(f)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_all(pool: &Pool) -> Result<Option<Vec<DbFeed>>, DbError> {
        let conn = pool.get()?;

        match db_feeds.load::<DbFeed>(&conn) {
            Ok(v) => Ok(Some(v)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

impl DbSubscription {
    /// Add Feed to the db and returns its title
    pub async fn add(
        server_id: &str,
        channel_id: &str,
        url: &str,
        pool: &Pool,
    ) -> Result<String, MyError> {
        // TODO: Ged rid of this FeedError
        // Get feed canvas_id
        let feed = Feed::from_url(url).await?;

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

        // Insert subscription
        let _row_inserted = match diesel::insert_into(subscriptions::table)
            .values(&new_subscription)
            .execute(&conn)
        {
            Ok(n) => n,
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => return Err(DbError::UniqueViolation.into()),
            e => e?,
        };

        Ok(feed.title)
    }

    pub fn get_by_feed_id(feed_id: i32, pool: &Pool) -> Result<Option<Vec<Self>>, DbError> {
        let conn = pool.get()?;

        match db_subscriptions
            .filter(subscriptions::feed_id.eq(feed_id))
            .get_results(&conn)
        {
            Ok(v) => Ok(Some(v)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
