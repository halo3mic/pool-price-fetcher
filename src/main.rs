mod cli;

use std::path::{PathBuf, Path};
use std::ops::Range;
use eyre::Result;
use uuid_b64::UuidB64;
use pool_price_fetcher::{
    PriceFetcherBuilder,
    PriceFetcherResult,
    ChainConfig,
    Config,
    self,
};


#[tokio::main]
async fn main() -> Result<()> {
    match cli::parse_cli_args() {
        cli::Commands::FetchPrices(args) => handle_fetch_prices_command(args).await?,
    }
    Ok(())
}


const DEFAULT_CONFIG_PATH: &str = "./config.toml";
const DEFAULT_DATA_DIR: &str = "./.data";

async fn handle_fetch_prices_command(cli_args: cli::FetchPricesArgs) -> Result<()> {
    let config_path = cli_args.config_file_path
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH));
    let config = Config::try_from_file(&config_path)?;
    let precision = config.precision;
    let chain_config = config.chain_configs
        .into_iter()
        .find(|chain_config| {
            chain_config.chain_id == cli_args.chain_id
        })
        .ok_or_else(|| eyre::eyre!("Chain with ID {} not found in config", cli_args.chain_id))?;

    let write_dir = cli_args.write_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATA_DIR));
    let label = cli_args.label
        .unwrap_or_else(|| format!("{}_{}", chain_config.chain_id, UuidB64::new().to_string()));

    fetch_and_write_prices(
        chain_config,
        precision,
        cli_args.block_range.into(),
        &write_dir,
        &label,
    ).await?;
    
    Ok(())
}

async fn fetch_and_write_prices(
    chain_config: ChainConfig,
    precision: u8,
    block_range: Range<u64>,
    write_dir: &PathBuf,
    label: &str,
) -> Result<()> {
    let chain_id = chain_config.chain_id;
    let sources = chain_config.price_sources.iter()
        .map(|source| source.protocol.clone().into_boxed().name())
        .collect::<Vec<_>>();
    let prices = fetch_prices_for_chain(
        chain_config,
        precision,
        block_range.clone()
    ).await?;

    let write_dir = write_dir.join(label);
    create_dir(&write_dir)?;
    write_prices(prices, &write_dir)?;
    write_metadata(chain_id, sources, precision, block_range, &write_dir)?;
    Ok(())
}

async fn fetch_prices_for_chain(
    chain_config: ChainConfig,
    precision: u8,
    block_range: Range<u64>,
) -> Result<Vec<PriceFetcherResult>> {
    let price_fetcher = PriceFetcherBuilder::default()
        .precision(precision)
        .reth_db_path(&chain_config.reth_db_path)
        .rpc_url(chain_config.rpc_url)
        .price_sources(chain_config.price_sources)
        .build()
        .await?;
    price_fetcher.fetch_prices(block_range)
}

fn write_prices(prices: Vec<PriceFetcherResult>, write_dir: &PathBuf) -> Result<()> {
    let out_path = write_dir.join("data.parquet");
    pool_price_fetcher::writer::write_prices_to_parquet(&prices, &out_path)?;
    println!("Prices written to: {}", out_path.display());
    Ok(())
}

fn write_metadata(
    chain_id: u64,
    sources: Vec<String>,
    precision: u8,
    block_range: Range<u64>,
    write_dir: &PathBuf,
) -> Result<()> {
    let out_path = write_dir.join("metadata.json");
    let metadata = pool_price_fetcher::PricesMetadata {
        chain_id: chain_id,
        start_block: block_range.start,
        end_block: block_range.end,
        precision,
        sources,
    };
    
    pool_price_fetcher::writer::write_prices_metadata(metadata, &out_path)?;
    println!("Metadata written to: {}", out_path.display());
    Ok(())
}

fn create_dir(path: &Path) -> Result<()> {    
    if path.exists() {
        Err(eyre::eyre!("Target {:?} already exists", path))
    } else {
        std::fs::create_dir(path)?;
        Ok(())
    }
}