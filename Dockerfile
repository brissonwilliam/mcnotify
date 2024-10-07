FROM rust:1.81 AS builder

COPY . ./src
WORKDIR /src

RUN cargo build --release --bin mcnotify 

FROM alpine:3.20
COPY --from=builder /src/target/release/mcnotify /mcnotify

ENTRYPOINT ["mcnotify"]
