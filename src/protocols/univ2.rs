use alloy::primitives::{Address, B256, U256, uint, b256};
use eyre::Result;
use super::common::{self, Protocol};


const U112_MASK: U256 = uint!(5192296858534827628530496329220095_U256);
const UNIV2_RESERVES_SLOT: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000008");

#[derive(serde::Deserialize, Debug, Clone)]
pub struct UniV2 {
    pool: Address,
}

#[async_trait::async_trait]
impl Protocol for UniV2 {

    fn name(&self) -> String {
        format!("UniV2: {}", self.pool)
    }

    fn storage_slot(&self) -> B256 {
        UNIV2_RESERVES_SLOT
    }

    fn storage_target(&self) -> Address {
        self.pool
    }

    fn retrieve_price_from_storage(
        &self,
        storage: U256,
        inverse_it: bool,
        dec_denoms: [U256; 2],
        precision_factor: U256,
    ) -> Result<U256> {
        let token0_reserve = storage & U112_MASK;
        let token1_reserve = storage >> 112 & U112_MASK;
        let price = 
            if inverse_it {
                precision_factor * token0_reserve * dec_denoms[1] / (token1_reserve * dec_denoms[0])
            } else {
                precision_factor * token1_reserve * dec_denoms[0] / (token0_reserve * dec_denoms[1])
            };

        Ok(price)
    }

    async fn fetch_tokens(
        &self,
        provider: &alloy::providers::RootProvider,
    ) -> Result<[Address; 2]> {
        common::uniswap_pool_tokens(provider, self.pool).await
    }
}
