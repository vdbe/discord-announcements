#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

pub use error::{DbError, FeedError, MyError};
pub use models::{DbSubscription, Feed};

mod error;
mod models;
mod schema;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
