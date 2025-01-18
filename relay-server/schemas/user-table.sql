create table users (
    id integer primary key autoincrement,
    username varchar unique,
    ip integer unique,
    token varchar(16) unique
);
