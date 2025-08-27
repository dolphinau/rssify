FROM rust:1.88

WORKDIR /src
COPY . .

RUN cargo build --release
CMD ["./target/release/rssify-cli", "/rss/"]
