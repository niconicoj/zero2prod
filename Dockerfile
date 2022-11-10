FROM lukemathwalker/cargo-chef:latest-rust-alpine as chef
WORKDIR /app
RUN apk add --no-cache musl-dev openssl-dev

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin zero2prod

FROM alpine:latest as runtime

RUN apk add --no-cache openssl-dev
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY config.toml config.toml

EXPOSE 3000

ENTRYPOINT [ "./zero2prod" ]
