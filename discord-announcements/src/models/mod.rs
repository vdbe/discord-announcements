mod canvas;
mod db;

use db::{DbFeed, NewFeed};

pub use canvas::Feed;
pub use db::DbSubscription;

pub struct Channel {
    server_id: String,
    channel_id: String,
}

impl Channel {
    pub fn new(server_id: String, channel_id: String) -> Self {
        Self {
            server_id,
            channel_id,
        }
    }
}
