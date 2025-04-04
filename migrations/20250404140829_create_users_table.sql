-- Add migration script here
-- 创建用户表
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL
);

-- 插入一些测试数据
INSERT INTO users (username, email) VALUES 
    ('测试用户1', 'test1@example.com'),
    ('测试用户2', 'test2@example.com');