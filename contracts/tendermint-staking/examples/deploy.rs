use abstract_boot::ApiDeployer;

use abstract_boot::boot_core::networks::{parse_network, NetworkInfo};
use abstract_boot::boot_core::*;
use abstract_tendermint_staking_api::boot::TMintStakingApi;
use abstract_tendermint_staking_api::TENDERMINT_STAKING;
use cosmwasm_std::{Decimal, Empty};
use semver::Version;
use std::sync::Arc;
use tokio::runtime::Runtime;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_tendermint_staking(network: NetworkInfo) -> anyhow::Result<()> {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;
    let mut staking = TMintStakingApi::new(TENDERMINT_STAKING, chain);
    staking.deploy(
        version,
        Empty {},
    )?;
    Ok(())
}

use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long)]
    network_id: String,
}

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let args = Arguments::parse();

    let network = parse_network(&args.network_id);

    deploy_tendermint_staking(network)
}
