use std::path::Path;
use std::sync::Arc;
use eyre::Result;

use reth_ethereum::node::{api::NodeTypesWithDBAdapter, EthereumNode};
use reth_ethereum::chainspec::ChainSpecBuilder;
use reth_ethereum::provider::{
    db::{mdbx::DatabaseArguments, open_db_read_only, ClientVersion, DatabaseEnv},
    providers::StaticFileProvider,
    ProviderFactory,
};

pub type LocalProviderFactory = ProviderFactory<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>;

pub fn build_provider_factory(db_path: &Path) -> Result<LocalProviderFactory> {
    let db_path = Path::new(&db_path);
    let db = Arc::new(open_db_read_only(
        db_path.join("db").as_path(),
        DatabaseArguments::new(ClientVersion::default()),
    )?);
    let spec = Arc::new(ChainSpecBuilder::mainnet().build());
    let factory = ProviderFactory::<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>::new(
        db.clone(),
        spec.clone(),
        StaticFileProvider::read_only(db_path.join("static_files"), true)?,
    );
    Ok(factory)
}

