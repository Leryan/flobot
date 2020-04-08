-- Your SQL goes here
CREATE TABLE blague (
    id integer primary key not null,
    team_id varchar(256) not null,
    text text not null,
    UNIQUE(team_id, text)
);