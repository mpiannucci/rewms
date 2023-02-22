FROM rust:1.67 as build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/rewms

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src src/

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian11

COPY --from=build /usr/local/cargo/bin/rewms /usr/local/bin/rewms

ENTRYPOINT ["rewms"]