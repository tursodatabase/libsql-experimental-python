CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    name TEXT,
    age INTEGER
);

INSERT INTO users (name, age) VALUES ('Alice', 25);
INSERT INTO users (name, age) VALUES ('Bob', 30);

SELECT * FROM users;

UPDATE users SET age = 26 WHERE name = 'Alice';

SELECT * FROM users;
