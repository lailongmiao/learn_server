-- Add migration script here
alter table groups add column team_id int references teams(id);

update groups set team_id=1 where id in(1,2);
update groups set team_id=2 where id in(3,4);

alter table groups alter column team_id set not null;