-- Your SQL goes here
CREATE TABLE trigger_new (
    id integer primary key not null,
    triggered_by varchar(256) not null,
    emoji varchar(256),
    text_ varchar(256),
    UNIQUE(triggered_by, emoji, text_)
);
INSERT INTO trigger_new SELECT * FROM trigger;
DROP TABLE trigger;
ALTER TABLE trigger_new RENAME TO trigger;
ALTER TABLE trigger ADD COLUMN team_id varchar(256) not null default '';