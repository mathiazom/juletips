FROM rust:1.100-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release
COPY . .
# COPY keeps the context's mtimes, which can predate the stub build
RUN touch src/main.rs && cargo build --release && cp target/release/juletips /juletips

FROM scratch
COPY --from=build /juletips /juletips
VOLUME ["/data/db", "/data/site"]
EXPOSE 8080
ENTRYPOINT ["/juletips"]
