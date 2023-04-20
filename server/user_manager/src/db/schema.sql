CREATE DATABASE user_db;

CREATE TABLE account (
     user_id serial PRIMARY KEY,
     username VARCHAR ( 500 ) UNIQUE NOT NULL,
     full_name VARCHAR ( 500 ) NOT NULL,
     password VARCHAR ( 500 ) NOT NULL,
     email VARCHAR ( 500 ) UNIQUE NOT NULL,
     phone VARCHAR ( 100 ) UNIQUE NOT NULL
);
