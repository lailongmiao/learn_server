-- Add migration script here
alter table users alter column team_id set not null;
alter table users alter column group_id set not null;