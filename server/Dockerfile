FROM rust:1.63

RUN USER=root cargo new --bin rust_server
WORKDIR /rust_server

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
COPY ./config ./config

RUN rm ./target/release/deps/server*
RUN cargo install --path .

EXPOSE 8080

CMD ["server"]