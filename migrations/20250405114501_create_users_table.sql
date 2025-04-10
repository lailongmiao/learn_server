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

INSERT INTO users(id, username, email, team_id, group_id,password) VALUES
    (1, '用户测试1', 'test1@example.com', 1, 1,'123456'),
    (2, '用户测试2', 'test2@example.com', 1, 1,'123456'),
    (3, 'test3', 'test3@example.com', 2, 3,'123456'),
    (4, 'test4', 'test4@example.com', 2, 3,'123456'),
    (5, 'test5', 'test5@example.com', 2, 4,'123456'),
    (6, 'test6', 'test6@example.com', 2, 4,'123456'),
    (7, '测试用户7', 'test7@example.com', 1, 2,'123456'),
    (8, '测试用户8', 'test8@example.com', 1, 2,'123456');

