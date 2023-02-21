FROM rust:1.67

WORKDIR /usr/src/rewms

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src src/

RUN cargo install --path .

CMD ["rewms"]