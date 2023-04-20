CREATE DATABASE order_db;


CREATE TABLE orders (
    order_id serial PRIMARY KEY,
    user_id INT NOT NULL,
    product_id INT NOT NULL,
    date_time TIMESTAMP NOT NULL,
    total_price DECIMAL NOT NULL
);
