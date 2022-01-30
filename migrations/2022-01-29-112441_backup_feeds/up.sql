-- Your SQL goes here
CREATE TABLE backup_feeds (
	id		SERIAL PRIMARY KEY,
	feed_id		INTEGER NOT NULL REFERENCES feeds (id),
	url		VARCHAR UNIQUE NOT NULL
);
