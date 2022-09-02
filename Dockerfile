FROM ekidd/rust-musl-builder as cargo-build
WORKDIR /home/rust/src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src ./src
COPY ./i18n ./i18n
COPY ./i18n.toml ./i18n.toml

FROM cargo-build as release-dev
RUN cargo build
RUN cargo test

FROM cargo-build as release-prod
RUN cargo build --release

FROM alpine:latest as image-dev
COPY --from=release-dev /home/rust/src/target/x86_64-unknown-linux-musl/debug/gitlab-project-doctor /usr/local/bin
RUN gitlab-project-doctor --help

FROM alpine:latest as image-prod
COPY --from=release-prod /home/rust/src/target/x86_64-unknown-linux-musl/release/gitlab-project-doctor /usr/local/bin
