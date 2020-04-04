-- Your SQL goes here
CREATE TABLE edits (
    id integer primary key not null,
    edit varchar(256) not null,
    team_id varchar(256),
    user_id varchar(256),
    replace_with_text text,
    replace_with_file varchar(256),
    UNIQUE(edit, team_id, user_id)
);
