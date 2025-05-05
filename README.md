## Pool Price Fetcher

A tool for fetching historical price data from EVM pools. 

### Support
For now only UniswapV2 and UniswapV3 are supported.

### Requirements
Local RethDB

### Setup

1. Copy `config.sample.toml` to `config.toml` and customize
2. Replace environment variables in config with your values

### Example Usage

```bash
./target/release/pool-price-fetcher fetch-prices --chain-id 1 --block-range 12345678..12345900
```

Output is saved as Parquet files with price data and JSON metadata.