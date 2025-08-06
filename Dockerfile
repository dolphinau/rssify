FROM rust:1.88

WORKDIR /src
COPY . .

RUN cargo build --release
CMD ["./target/release/lwn-sub-snoozer", "/rss/lwn-sub.xml"]
