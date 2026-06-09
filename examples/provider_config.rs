//! Minimal provider-side configuration shape.
//!
//! This example does not start a server. It shows the public configuration
//! surface an inference provider would need before exposing MPP-I endpoints.

use std::env;

#[derive(Debug)]
struct ProviderConfig {
    chain_id: u64,
    escrow_contract: String,
    provider_wallet: String,
    price_per_input_token_wei: String,
    price_per_output_token_wei: String,
    quote_ttl_seconds: u64,
}

impl ProviderConfig {
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            chain_id: env::var("MPPI_CHAIN_ID")?.parse()?,
            escrow_contract: env::var("MPPI_ESCROW_CONTRACT")?,
            provider_wallet: env::var("MPPI_PROVIDER_WALLET")?,
            price_per_input_token_wei: env::var("MPPI_INPUT_TOKEN_PRICE_WEI")?,
            price_per_output_token_wei: env::var("MPPI_OUTPUT_TOKEN_PRICE_WEI")?,
            quote_ttl_seconds: env::var("MPPI_QUOTE_TTL_SECONDS")?.parse()?,
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ProviderConfig::from_env()?;

    println!("chain_id={}", config.chain_id);
    println!("escrow_contract={}", config.escrow_contract);
    println!("provider_wallet={}", config.provider_wallet);
    println!("input_price_wei={}", config.price_per_input_token_wei);
    println!("output_price_wei={}", config.price_per_output_token_wei);
    println!("quote_ttl_seconds={}", config.quote_ttl_seconds);

    Ok(())
}
