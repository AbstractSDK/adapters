pub(crate) mod commands;
pub mod contract;
pub(crate) mod dex_trait;
pub mod error;
mod exchanges;
pub mod msg;

pub mod api;
pub(crate) mod handlers;
pub mod state;

pub use commands::LocalDex;
pub use dex_trait::DexCommand;

pub const EXCHANGE: &str = "abstract:dex";

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub mod host_exchange {
    pub use super::exchanges::osmosis::Osmosis;
}

#[cfg(feature = "boot")]
pub mod boot {
    use crate::{msg::*, EXCHANGE};
    use abstract_boot::boot_core::ContractWrapper;
    use abstract_boot::boot_core::{contract, Contract, ContractInstance, CwEnv};
    use abstract_boot::{AbstractBootError, AdapterDeployer, Manager};
    use abstract_core::{
        adapter::{self},
        objects::{AnsAsset, AssetEntry},
        MANAGER,
    };
    use cosmwasm_std::{Decimal, Empty};

    #[contract(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct DexAdapter<Chain>;

    // Implement deployer trait
    impl<Chain: CwEnv> AdapterDeployer<Chain, DexInstantiateMsg> for DexAdapter<Chain> {}

    impl<Chain: CwEnv> DexAdapter<Chain> {
        pub fn new(name: &str, chain: Chain) -> Self {
            Self(
                Contract::new(name, chain)
                    .with_wasm_path("abstract_dex_adapter")
                    .with_mock(Box::new(ContractWrapper::new_with_empty(
                        crate::contract::execute,
                        crate::contract::instantiate,
                        crate::contract::query,
                    ))),
            )
        }

        /// Swap using Abstract's OS (registered in daemon_state).
        pub fn swap(
            &self,
            offer_asset: (&str, u128),
            ask_asset: &str,
            dex: String,
        ) -> Result<(), AbstractBootError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());
            let asset = AssetEntry::new(offer_asset.0);
            let ask_asset = AssetEntry::new(ask_asset);

            let swap_msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: DexExecuteMsg::Action {
                    dex,
                    action: DexAction::Swap {
                        offer_asset: AnsAsset::new(asset, offer_asset.1),
                        ask_asset,
                        max_spread: Some(Decimal::percent(30)),
                        belief_price: None,
                    },
                },
            });
            manager.execute_on_module(EXCHANGE, swap_msg)?;
            Ok(())
        }
    }
}
