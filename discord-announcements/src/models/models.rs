use diesel::{QueryDsl, RunQueryDsl};
use reqwest::IntoUrl;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

mod canvas;
mod db;
