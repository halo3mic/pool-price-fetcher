use alloy::primitives::{Address, B256, U256, uint};
use eyre::Result;
use super::common::{self, Protocol};


const TWO_POW_96: U256 = uint!(79228162514264337593543950336_U256);
const U160_MASK: U256 = uint!(1461501637330902918203684832716283019655932542975_U256);
const UNIV3_SQRT_PRICE_X96_SLOT: B256 = B256::ZERO;
const E30: U256 = uint!(1000000000000000000000000000000_U256);
const E15: U256 = uint!(1000000000000000_U256);

#[derive(serde::Deserialize, Debug, Clone)]
pub struct UniV3 {
    pool: Address,
}

#[async_trait::async_trait]
impl Protocol for UniV3 {
    fn name(&self) -> String {
        format!("UniV3: {}", self.pool)
    }

    fn storage_slot(&self) -> B256 {
        UNIV3_SQRT_PRICE_X96_SLOT
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
        let sqrt_price_x96 = storage & U160_MASK;
        let price = ((E15 * sqrt_price_x96 / TWO_POW_96) as U256).pow(U256::from(2));
        
        let price = if inverse_it {
            precision_factor * E30 * dec_denoms[1] / (price * dec_denoms[0])
        } else {
            precision_factor * price * dec_denoms[0] / dec_denoms[1] / E30
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