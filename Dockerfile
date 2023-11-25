FROM alpine:3 as builder

WORKDIR /usr/src/zero2prod

# Install dependencies
RUN apk add --no-cache gcc musl-dev pkgconfig openssl-dev
RUN apk add --no-cache rust cargo
COPY . .
ENV SQLX_OFFLINE true
RUN cargo install --path ./zero2prod

FROM alpine:latest

COPY --from=builder /root/.cargo/bin/zero2prod /usr/local/bin/zero2prod

EXPOSE 8000

CMD ["zero2prod"]
