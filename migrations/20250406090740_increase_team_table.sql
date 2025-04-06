-- Add migration script here
insert into users(id,username,email) values (4,'test4','test4@example.com');

create table teams
(
    id serial primary key,
    name varchar(255) not null
);
insert into teams(name) values('team1'),('team2');

alter table users add column team_id int references teams(id);

update users set team_id=1 where id=1 or id=2;
update users set team_id=2 where id=3 or id=4;

