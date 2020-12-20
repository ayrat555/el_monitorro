FROM rust:latest

WORKDIR /app

RUN apt -qq  update

RUN apt -qq install -y  build-essential sudo \
    cargo curl bash postgresql

RUN rustup install nightly

RUN rustup override set nightly

COPY . .

RUN cargo build --release

RUN cargo install diesel_cli

EXPOSE 5432

RUN diesel database setup

RUN diesel migration run

CMD ["bash", "/app/src/start.sh"]
