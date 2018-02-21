FROM rust:latest as builder
WORKDIR /rust
COPY . .
RUN cargo install -v

FROM debian:stretch-slim
# RUN apk update && apk add ca-certificates && rm -rf /var/cache/apk/*
COPY --from=builder /usr/local/cargo/bin/service /service
ENTRYPOINT [ "/service" ]