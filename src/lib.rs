mod price_fetcher;
mod protocols;
mod config;
mod reth_utils;
pub mod writer;

pub use price_fetcher::{PriceFetcherBuilder, PriceFetcher, PriceFetcherResult};
pub use config::{Config, ChainConfig};

#[derive(serde::Serialize, Debug)]
pub struct PricesMetadata {
    pub chain_id: u64, 
    pub start_block: u64,
    pub end_block: u64,
    pub sources: Vec<String>,
    pub precision: u8,
}