-- Your SQL goes here
CREATE TABLE subscriptions (
	id		SERIAL PRIMARY KEY,
	server_id	VARCHAR NOT NULL,
	channel_id	VARCHAR NOT NULL,
	feed_id		INTEGER NOT NULL REFERENCES feeds (id),
	UNIQUE (server_id, channel_id, feed_id)
);

