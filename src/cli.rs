use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;
use std::ops::Range;


#[derive(Debug, Clone)]
pub struct BlockRange {
    start: u64,
    end: u64,
}

impl FromStr for BlockRange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("..").collect();
        if parts.len() != 2 {
            return Err("Block range must be in format 'start..end'".to_string());
        }

        let start = parts[0]
            .parse::<u64>()
            .map_err(|_| "Failed to parse start block as u64".to_string())?;
        let end = parts[1]
            .parse::<u64>()
            .map_err(|_| "Failed to parse end block as u64".to_string())?;

        if start >= end {
            return Err("Start block must be less than end block".to_string());
        }

        Ok(BlockRange { start, end })
    }
}

impl From<BlockRange> for Range<u64> {
    fn from(range: BlockRange) -> Self {
        range.start..range.end
    }
}

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    FetchPrices(FetchPricesArgs),
}

#[derive(Args)]
pub struct FetchPricesArgs {
    #[arg(long, default_value = "1")]
    pub chain_id: u64,

    #[arg(value_parser = BlockRange::from_str)]
    pub block_range: BlockRange,

    #[arg(long)]
    pub config_file_path: Option<PathBuf>,

    #[arg(long)]
    pub write_dir: Option<PathBuf>,

    #[arg(long)]
    pub label: Option<String>,
}

pub fn parse_cli_args() -> Commands {
    Cli::parse().command
}
