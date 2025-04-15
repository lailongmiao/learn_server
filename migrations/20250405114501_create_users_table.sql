CREATE TABLE teams (
    id uuid primary key DEFAULT gen_random_uuid(),
    name varchar(255) not null
);


CREATE TABLE groups (
    id uuid primary key DEFAULT gen_random_uuid(),
    name varchar(255) not null,
    team_id uuid references teams(id) not null
);


CREATE TABLE users (
    id uuid primary key DEFAULT gen_random_uuid(),
    username varchar(255) not null unique,
    primary_email_address varchar(255) not null unique,
    team_id uuid references teams(id),
    group_id uuid references groups(id),
    password varchar(255) not null
);


INSERT INTO teams(id, name) VALUES
    ('a1111111-1111-1111-1111-111111111111', 'team1'),
    ('a2222222-2222-2222-2222-222222222222', 'team2');

INSERT INTO groups(id, name, team_id) VALUES
    ('b1111111-1111-1111-1111-111111111111', 'Group A', 'a1111111-1111-1111-1111-111111111111'),
    ('b2222222-2222-2222-2222-222222222222', 'Group B', 'a1111111-1111-1111-1111-111111111111'),
    ('b3333333-3333-3333-3333-333333333333', 'Group C', 'a2222222-2222-2222-2222-222222222222'),
    ('b4444444-4444-4444-4444-444444444444', 'Group D', 'a2222222-2222-2222-2222-222222222222');
