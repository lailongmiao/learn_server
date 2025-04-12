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

INSERT INTO users(id, username, email, team_id, group_id, password) VALUES
    (1, '用户测试1', 'test1@example.com', 1, 1, '$argon2id$v=19$m=19456,t=2,p=1$VL9XcbM9aFmDnHpnr7ynUQ$MsK6XJfxR0/ek33D7O4r6rEJDMlIE2c+Gk5HGLo9AIM'),
    (2, '用户测试2', 'test2@example.com', 1, 1, '$argon2id$v=19$m=19456,t=2,p=1$ZwRWwu1PpH4TDY0I4UKM6g$OZPJqg5v/R8nUZh3iiRGELu2bVKJ0/LNZUI95xRhn+A'),
    (3, 'test3', 'test3@example.com', 2, 3, '$argon2id$v=19$m=19456,t=2,p=1$wKEHIwdBzwYeG9z3vkUNGQ$hVwH8urJKwDd7TYeU95K9+P1gdrYTuLqcxjxh2o3YxM'),
    (4, 'test4', 'test4@example.com', 2, 3, '$argon2id$v=19$m=19456,t=2,p=1$DgC0kqrKXnQMMK+oBlDOxg$efrWlGBYMJnG6OCt+p0yxY8z2VXH8eoOKYgHnw7B78s'),
    (5, 'test5', 'test5@example.com', 2, 4, '$argon2id$v=19$m=19456,t=2,p=1$hRXzVHiKSGfylbIEWXaydA$jUFyCe3zqmdiN9kcVFnqaCQoDXNmdjbYBY7wefpuG6Q'),
    (6, 'test6', 'test6@example.com', 2, 4, '$argon2id$v=19$m=19456,t=2,p=1$pxhEVeGTHnEJjg4vURK6Kw$WY7SaLBL8VADSgvtGZCgL5h8Ks82hhk8KBw2qOl8AXw'),
    (7, '测试用户7', 'test7@example.com', 1, 2, '$argon2id$v=19$m=19456,t=2,p=1$dLo2YosSdpV9J7NX+t+l7A$Q3PBZy3eMq8sOCnJ++BXMGGHvKj0kE7y7nFfEXOBtfM'),
    (8, '测试用户8', 'test8@example.com', 1, 2, '$argon2id$v=19$m=19456,t=2,p=1$T9KkI+uPsZJrAbP1+YN2IA$JnWmJXCLLa6S0YPXmG5yfLnBkmuYRKXOgJeYzKNR5vo');

