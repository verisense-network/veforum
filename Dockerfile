FROM rust:1-slim-bullseye

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        libclang-dev g++

WORKDIR /usr/src/vesync

COPY . .

RUN cargo install --path vesync/

CMD ["/usr/local/cargo/bin/vesync"]
