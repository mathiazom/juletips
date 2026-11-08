FROM rust:1.99-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release && cp target/release/juletips /juletips

FROM scratch
COPY --from=build /juletips /juletips
VOLUME ["/data/db", "/data/site"]
EXPOSE 8080
ENTRYPOINT ["/juletips"]
