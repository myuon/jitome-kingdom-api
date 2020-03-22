FROM rust:1.42-slim-stretch AS build

WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev pkg-config default-libmysqlclient-dev
COPY ./Cargo.toml /app
COPY ./Cargo.lock /app
RUN mkdir -p /app/src && echo "fn main() {}" > /app/src/main.rs
RUN cargo build --release

# remove build cache
RUN rm -r /app/target/release/.fingerprint/*jitome*

COPY ./ /app
RUN cargo build --release

RUN cp /app/target/release/jitome-kingdom-api /app/main
RUN chmod +x /app/main

EXPOSE 7999

CMD ["/app/main"]
