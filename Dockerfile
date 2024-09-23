# 1. Build
FROM rust:1.79.0 AS builder
RUN rustup target add wasm32-unknown-unknown
WORKDIR /build

COPY . .
RUN cargo build -p filler --release

# 2. Run
FROM gcr.io/distroless/cc-debian12 as runtime
WORKDIR /root

COPY --from=builder /build/target/release/filler .
COPY ./config.testnet.json ./config.testnet.json

ENV RUST_LOG="info" \
    WALLET_MNEMONIC="your mnemonic"  \
    COINGECKO_API_KEY="your api key" \
    TRADERS_OFFSET=0

CMD ["./filler"]
