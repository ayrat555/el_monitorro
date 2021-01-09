FROM rustlang/rust:nightly-alpine3.12

RUN apk add openssl-dev build-base postgresql-dev

WORKDIR /app

COPY . .

RUN cargo build --release
