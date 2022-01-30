use diesel::{QueryDsl, RunQueryDsl};
use reqwest::IntoUrl;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::diesel::ExpressionMethods;
use crate::error::MyError;
use crate::schema::feeds as schema_feeds;
use crate::schema::feeds::dsl::feeds as db_feeds;
use crate::Pool;

use super::{DbFeed, NewFeed};

mod rfc3339_time {
    use serde::{Deserialize, Deserializer};
    use std::time::SystemTime;
    use time::format_description::well_known::Rfc3339;
    use time::OffsetDateTime;

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<SystemTime, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;

        // TODO: Get rid of this unwrap
        Ok(OffsetDateTime::parse(&time, &Rfc3339).unwrap().into())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feed {
    pub xmlns: String,

    /// Url of course announcements on canvas
    pub id: String,

    pub title: String,

    /// RSS was updated on `updated`
    #[serde(with = "rfc3339_time")]
    pub updated: SystemTime,

    /// Same value as `id`
    pub link: Link,

    /// All announcements
    #[serde(rename = "entry")]
    pub announcements: Vec<Announcement>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Announcement {
    /// Title of announcement
    pub title: String,

    /// No idea what we could use this for
    /// format:
    ///     tag:<host?>:<date placed>:/<url path in course>
    pub id: String,

    /// Last time announcement was updated (on canvas)
    #[serde(with = "rfc3339_time")]
    pub updated: SystemTime,

    /// When the announcement was placed
    //#[serde(with = "rfc3339_time")]
    //pub published: OffsetDateTime,
    #[serde(with = "rfc3339_time")]
    pub published: SystemTime,

    /// Link to the the announcements
    /// TODO: Check if there can be more than 1 <link/> in an entry
    pub link: Link,

    /// Person that made the announcements
    pub author: Author,

    /// Content
    pub content: Content,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    /// rel
    pub rel: String,

    /// href
    pub href: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Author {
    /// Author: Firstame Lastname
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Content {
    /// content type: html, ...
    #[serde(rename = "type")]
    pub content_type: String,

    /// html of the announcements itself
    #[serde(rename = "$value")]
    pub content: String,
}

impl Feed {
    /// Get a feed from a atom url
    pub async fn from_url<T: IntoUrl>(url: T) -> Result<Feed, MyError> {
        let body = reqwest::get(url).await?.text().await?;

        Ok(quick_xml::de::from_str::<Feed>(&body)?)
    }

    /// only keep announcements published after `after`
    ///
    /// If there are none returns `None`,
    /// otherwise returns publication `SystemTime` of latest `Announcement`
    pub fn after(&mut self, after: SystemTime) -> Option<SystemTime> {
        self.announcements
            .retain(|announcement| announcement.published > after);

        self.announcements
            .iter()
            .max_by_key(|f| f.published)
            .map(|f| f.published)
    }

    /// Add a feed to the db
    pub async fn add(feed_url: &str, pool: &Pool) -> Result<(), MyError> {
        // Check if feed url is valid
        let feed = Feed::from_url(feed_url).await;
        if feed.is_err() {
            return Err(MyError::new("Cold not deserialize xml file"));
        }

        // Not an error checked above
        let feed = feed.unwrap();

        let conn = pool.get()?;

        // Check if feed id already exists
        match diesel::select(diesel::dsl::exists(
            db_feeds.filter(schema_feeds::canvas_id.eq(&feed.id)),
        ))
        .get_result(&conn)
        {
            Ok(true) => {
                match diesel::select(diesel::dsl::exists(
                    db_feeds.filter(schema_feeds::url.eq(feed_url)),
                ))
                .get_result(&conn)
                {
                    Ok(true) => (), // Exact feed already exists
                    Ok(false) => {
                        // Add to backup feeds
                        todo!();
                    }
                    Err(_) => todo!(), // Error
                }
            }
            Ok(false) => {
                // Add to db
                let new_feed = NewFeed {
                    canvas_id: &feed.id,
                    url: feed_url,
                    last_update: UNIX_EPOCH,
                };

                // returns rows changes/inserted
                let _ret: usize = diesel::insert_into(schema_feeds::table)
                    .values(&new_feed)
                    .execute(&conn)?;

                return Ok(());
            }
            Err(_) => todo!(),
        }

        Err(MyError::new("blablabla"))
    }

    /// Retrieve all feeds
    pub async fn get_all(pool: &Pool) -> Result<Option<Vec<Self>>, MyError> {
        let vec_db_feeds = match DbFeed::get_all(pool) {
            Ok(Some(vec)) => vec,
            Ok(None) => return Ok(None),
            //Err(DieselError::NotFound) => return Ok(None), // TODO: Return Ok(None) when we have a proper error type
            Err(err) => return Err(err.into()),
        };

        // spawn tasks to retrieve the xml feeds
        let tasks: Vec<_> = vec_db_feeds
            .iter()
            .map(|item| tokio::spawn(Feed::from_url(item.url.to_owned())))
            .collect();

        // collect tasks
        let mut feeds: Vec<Feed> = Vec::new();
        for task in tasks {
            feeds.push(task.await??);
        }

        Ok(Some(feeds))
    }

    /// Retrieve feeds containing only announcements placed after the last time this function was called
    pub async fn get_new(pool: &Pool) -> Result<Option<Vec<Self>>, MyError> {
        let vec_db_feeds = match DbFeed::get_all(pool) {
            Ok(Some(vec)) => vec,
            Ok(None) => return Ok(None),
            //Err(DieselError::NotFound) => return Ok(None), // TODO: Return Ok(None) when we have a proper error type
            Err(err) => return Err(err.into()),
        };

        // spawn tasks to retrieve the xml feeds
        let tasks: Vec<_> = vec_db_feeds
            .iter()
            .map(|item| tokio::spawn(Feed::from_url(item.url.to_owned())))
            .collect(); // NOTE: .collect() needed otherwise start on .await

        let conn = pool.get()?;

        // collect tasks
        let mut vec_feeds: Vec<Feed> = Vec::new();
        for (i, task) in tasks.into_iter().enumerate() {
            let mut feed = task.await??;

            // Update the `last_update` table field of feeds if there is a announcement
            if let Some(last) = feed.after(vec_db_feeds[i].last_update) {
                diesel::update(db_feeds.filter(schema_feeds::canvas_id.eq(&feed.id)))
                    .set(schema_feeds::last_update.eq(last))
                    .execute(&conn)?;
            };

            vec_feeds.push(feed);
        }

        Ok(Some(vec_feeds))
    }
}
