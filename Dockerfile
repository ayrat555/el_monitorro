FROM rustlang/rust:nightly-bullseye

WORKDIR /app

COPY ./. .

RUN cargo install diesel_cli --no-default-features --features postgres

RUN cargo build --release

RUN cp ./target/release/cleaner ./cleaner

RUN cp ./target/release/sync ./sync

RUN cp ./target/release/el_monitorro ./el_monitorro

RUN cp ./target/release/deliver ./deliver

RUN rm -rf ./target

CMD ["bash", "/app/docker/start.sh"]
