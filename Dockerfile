####################################################################################################
## Builder
####################################################################################################
FROM rust:1.64.0-bullseye AS builder
RUN apt update && apt install -y libssl-dev pkg-config libz-dev libcurl4 postgresql
RUN update-ca-certificates

# Create appuser
ENV USER=bot
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /bot

COPY ./ .

RUN cargo install diesel_cli --no-default-features --features postgres

RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
FROM debian:bullseye-slim

RUN apt update && apt install -y postgresql

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /bot

# Copy our build
COPY --from=builder /bot/target/release/el_monitorro ./
COPY --from=builder /bot/target/release/deliver ./
COPY --from=builder /bot/target/release/sync ./
COPY --from=builder /bot/target/release/cleaner ./

COPY --from=builder /bot/docker/start.sh ./

COPY --from=builder /usr/local/cargo/bin/diesel ./
COPY --from=builder /bot/migrations/ ./migrations/


# Use an unprivileged user.
USER bot:bot

CMD ["bash", "/bot/start.sh"]
