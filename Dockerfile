FROM rustlang/rust:nightly-buster

WORKDIR /app

COPY ./. .

RUN cargo install diesel_cli --no-default-features --features postgres

RUN cargo build --release

CMD ["bash", "/app/docker/start.sh"]
