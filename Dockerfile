FROM rust:alpine as cargo-build

RUN apk update
RUN apk add --no-cache openssl-dev musl-dev

WORKDIR /home/rust/src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src ./src
COPY ./i18n ./i18n
COPY ./i18n.toml ./i18n.toml
RUN rustup target add x86_64-unknown-linux-musl

FROM cargo-build as release-dev
RUN cargo build --target x86_64-unknown-linux-musl
RUN cargo test

FROM cargo-build as release-prod
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:latest as image-dev
COPY --from=release-dev /home/rust/src/target/x86_64-unknown-linux-musl/debug/gitlab-project-doctor /usr/local/bin
RUN gitlab-project-doctor --help

FROM alpine:latest as image-prod
COPY --from=release-prod /home/rust/src/target/x86_64-unknown-linux-musl/release/gitlab-project-doctor /usr/local/bin
