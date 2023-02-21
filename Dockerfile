FROM rust:1.67-buster

RUN apt-get update && apt-get install -y \
    nginx

WORKDIR /usr/src/rewms

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src src/

RUN cargo install --path .

COPY nginx /

COPY ./scripts/run.sh /usr/local/bin

ENV DOWNSTREAM="http://localhost:8080"

CMD ["/bin/bash", "/usr/local/bin/run.sh"]