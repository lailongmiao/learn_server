CREATE TABLE teams (
    id serial primary key,
    name varchar(255) not null
);


CREATE TABLE groups (
    id serial primary key,
    name varchar(255) not null,
    team_id int references teams(id) not null
);


CREATE TABLE users (
    id serial primary key,
    username varchar(255) not null unique,
    email varchar(255) not null unique,
    team_id int references teams(id),
    group_id int references groups(id),
    password varchar(255) not null
);


INSERT INTO teams(name) VALUES
    ('team1'),
    ('team2');

INSERT INTO groups(name, team_id) VALUES
    ('Group A', 1),
    ('Group B', 1),
    ('Group C', 2),
    ('Group D', 2);

