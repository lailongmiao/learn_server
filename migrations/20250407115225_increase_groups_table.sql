-- Add migration script here
insert into users(id,username,email,team_id) values(5,'test5','test5@example.com',2),
(6,'test6','test6@example.com',2);
insert into users(id,username,email,team_id) values(7,'测试用户7','test7@example.com',1),
(8,'测试用户8','test8@example.com',1);

create table groups
(
    id serial primary key,
    name varchar(255) not null
);

insert into groups(name) values('Group A'),('Group B'),('Group C'),('Group D');

alter table users add column group_id int references groups(id);

update users set group_id=1 where id in(1,2);
update users set group_id=2 where id in(7,8);
update users set group_id=3 where id in(3,4);
update users set group_id=4 where id in(5,6);

