CREATE TABLE organizations
(
    id uuid primary key DEFAULT gen_random_uuid(),
    name varchar(255) not null
);


CREATE TABLE teams (
    id uuid primary key DEFAULT gen_random_uuid(),
    name varchar(255) not null,
    organization_id uuid references organizations(id) not null
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
    organization_id uuid references organizations(id),
    team_id uuid references teams(id),
    group_id uuid references groups(id),
    password varchar(255) not null
);
