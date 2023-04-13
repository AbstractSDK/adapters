mod common;

use std::env;
use std::sync::Arc;

use abstract_boot::boot_core::Deploy;
use abstract_boot::boot_core::{ContractInstance};
use abstract_boot::{Abstract, ApiDeployer};
use abstract_dex_api::{boot::DexApi, msg::DexInstantiateMsg, EXCHANGE};
use boot_core::networks::LOCAL_OSMO;
use boot_core::DaemonOptionsBuilder;
use cosmwasm_std::{coin, Addr, Decimal, Empty};

const MNEMONIC: &str = "quality vacuum heart guard buzz spike sight swarm shove special gym robust assume sudden deposit grid alcohol choice devote leader tilt noodle tide penalty";

#[test]
fn swap_native() -> anyhow::Result<()> {
    let rt = Arc::new(tokio::runtime::Runtime::new().unwrap());
    let options = DaemonOptionsBuilder::default()
        .network(LOCAL_OSMO)
        .build()?;
    env::set_var("LOCAL_MNEMONIC", MNEMONIC);

    let (sender, chain) = boot_core::instantiate_daemon_env(&rt, options)?;
    let wallet = chain.sender.clone();

    let deployment = Abstract::deploy_on(chain.clone(), "1.0.0".parse()?)?;

    deployment.account_factory.create_default_account(
        abstract_core::objects::gov_type::GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
    )?;
    let mut exchange_api = DexApi::new(EXCHANGE, chain.clone());

    exchange_api.deploy(
        "1.0.0".parse()?,
        DexInstantiateMsg {
            swap_fee: Decimal::percent(1),
            recipient_account: 0,
        },
    )?;

    

    let os = deployment.account_factory.create_default_account(
        abstract_core::objects::gov_type::GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
    )?;
    let proxy_addr = os.proxy.address()?;
    let _manager_addr = os.manager.address()?;
    // send to proxy
    rt.block_on(wallet.bank_send(proxy_addr.as_str(), vec![coin(10_000, "uosmo")]))?;
    // install exchange on OS
    os.manager.install_module(EXCHANGE, &Empty {})?;
    // load exchange data into type
    exchange_api.set_address(&Addr::unchecked(
        os.manager.module_info(EXCHANGE)?.unwrap().address,
    ));

    // swap 100 EUR to USD
    exchange_api.swap(("osmo", 100), "uion", "osmosis".into())?;

    // // check balances
    // let eur_balance = chain.query_balance(&proxy_addr, EUR)?;
    // assert_that!(eur_balance.u128()).is_equal_to(9_900);

    // let usd_balance = chain.query_balance(&proxy_addr, USD)?;
    // assert_that!(usd_balance.u128()).is_equal_to(98);

    // // assert that OS 0 received the swap fee
    // let os0_proxy = AbstractAccount::new(chain.clone(), Some(0))
    //     .proxy
    //     .address()?;
    // let os0_eur_balance = chain.query_balance(&os0_proxy, EUR)?;
    // assert_that!(os0_eur_balance.u128()).is_equal_to(1);

    Ok(())
}
