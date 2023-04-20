CREATE DATABASE product_db;

CREATE TABLE product (
     product_id serial PRIMARY KEY,
     name VARCHAR ( 500 ) UNIQUE NOT NULL,
     image VARCHAR ( 1000 ),
     count INT NOT NULL,
     price DECIMAL NOT NULL,
     category VARCHAR ( 500 ) NOT NULL
);
