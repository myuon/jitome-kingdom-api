FROM rust:1.42-slim-stretch AS build

WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev pkg-config default-libmysqlclient-dev
COPY ./Cargo.toml /app
COPY ./Cargo.lock /app
RUN mkdir -p /app/src && echo "fn main() {}" > /app/src/main.rs
RUN cargo build --release

RUN rm /app/src/main.rs
COPY ./src /app/src
COPY .env /app
RUN cargo build --release

RUN cp /app/target/release/jitome-kingdom-api /app/main

RUN useradd -u 50000 user
USER user

EXPOSE 7999

CMD ["/app/main"]
