FROM rust:1.51 as build

# create a new empty shell project
WORKDIR /usr/src/language-api
RUN cargo init --bin

COPY ./Cargo.lock ./Cargo.toml ./

# cache dependencies
RUN touch ./src/lib.rs
RUN cargo build --features=build-binary --release
RUN rm src/*.rs

# copy source tree, migrations, queries, sqlx data
COPY ./src ./src

# build for release, remove dummy compiled files (in workspace root)
RUN rm ./target/release/deps/*language_api*

RUN cargo build --features=build-binary --release --locked

## Final base image with only the picatch binary
FROM debian:buster-slim

WORKDIR /config

# target dir is still in workspace root
COPY --from=build /usr/src/language-api/target/release/language-api /usr/local/bin/language-api

EXPOSE 8080
ENTRYPOINT ["language-api"]
