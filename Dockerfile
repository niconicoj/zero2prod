FROM clux/muslrust:stable as chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine as runtime
RUN addgroup -S zero2prod && adduser -S zero2prod -G zero2prod
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/zero2prod /usr/local/bin/
COPY --from=builder /app/configuration configuration
USER zero2prod
CMD ["/usr/local/bin/zero2prod"]
