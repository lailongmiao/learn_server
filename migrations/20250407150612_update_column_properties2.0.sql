-- Add migration script here
alter table users alter column team_id drop not null;
alter table users alter column group_id drop not null;