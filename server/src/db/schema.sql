CREATE DATABASE online_shop;

CREATE TABLE account (
     user_id serial PRIMARY KEY,
     username VARCHAR ( 500 ) UNIQUE NOT NULL,
     full_name VARCHAR ( 500 ) NOT NULL,
     password VARCHAR ( 500 ) NOT NULL,
     email VARCHAR ( 500 ) UNIQUE NOT NULL,
     phone VARCHAR ( 100 ) UNIQUE NOT NULL
);


CREATE TABLE product (
     product_id serial PRIMARY KEY,
     name VARCHAR ( 500 ) UNIQUE NOT NULL,
     image VARCHAR ( 1000 ),
     count INT NOT NULL,
     price DECIMAL NOT NULL,
     category VARCHAR ( 500 ) NOT NULL
);


CREATE TABLE orders (
    order_id serial PRIMARY KEY,
    user_id INT NOT NULL,
    product_id INT NOT NULL,
    date_time TIMESTAMP NOT NULL,
    total_price DECIMAL NOT NULL,

    FOREIGN KEY (user_id)
        REFERENCES account (user_id),
    FOREIGN KEY (product_id)
        REFERENCES product (product_id)
);
