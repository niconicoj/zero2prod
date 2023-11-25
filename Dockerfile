FROM alpine:3 as builder

WORKDIR /usr/src/zero2prod

# Install dependencies
RUN apk add --no-cache gcc musl-dev pkgconfig openssl-dev
RUN apk add --no-cache rust cargo
COPY . .
ENV SQLX_OFFLINE true
RUN cargo install --path ./zero2prod

FROM alpine:3 as runtime

WORKDIR /app

RUN apk update
RUN apk add --no-cache gcc openssl-dev
COPY --from=builder /root/.cargo/bin/zero2prod /usr/local/bin/zero2prod
COPY --from=builder /usr/src/zero2prod/configuration.yaml configuration.yaml
COPY --from=builder /usr/src/zero2prod/configuration configuration

EXPOSE 8000

CMD ["zero2prod"]
