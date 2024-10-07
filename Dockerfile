FROM rust:1.81-slim AS builder

COPY . /src
WORKDIR /src

# to run under alpine, we have to target a special static link build profile
# and manually add openssl deps to build os (or use rust tls)
RUN rustup target add x86_64-unknown-linux-musl && \
    apt update && \
    apt install -y musl-tools musl-dev && \
    update-ca-certificates
# ENV OPENSSL_STATIC=1 
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.19
COPY --from=builder /src/target/x86_64-unknown-linux-musl/release/mcnotify /mcnotify

ENTRYPOINT [ "/mcnotify" ]
