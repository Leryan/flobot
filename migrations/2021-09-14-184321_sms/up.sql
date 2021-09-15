-- Your SQL goes here
CREATE TABLE sms_contact (
    id integer primary key not null,
    team_id varchar(256) not null,
    name varchar(256) not null,
    number varchar(256) not null,
    last_sending_unixts bigint not null,
    UNIQUE(team_id, name)
);

CREATE TABLE sms_prepare (
    id integer primary key not null,
    team_id varchar(256) not null,
    sms_contact_id integer not null,
    trigname varchar(256) not null,
    name varchar(256) not null,
    text text not null,
    UNIQUE(team_id, sms_contact_id, trigname),
    foreign key (sms_contact_id) references sms_contact(id) ON DELETE CASCADE
);