pub mod contract;
pub mod error;
mod lsds;
pub(crate) mod handlers;
pub mod msg;
pub mod state;
mod traits;
pub(crate) mod util;

// Export interface for use in SDK modules
pub use traits::api::{Dex, DexInterface};

pub const LSD: &str = "abstract:lsd";

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub mod host_exchange {
    pub use super::lsds::osmosis::Osmosis;
}

#[cfg(feature = "cw-orch")]
pub mod cw_orch {
    use abstract_interface::AbstractInterfaceError;
use abstract_interface::Manager;
    use abstract_interface::AdapterDeployer;
    use crate::{msg::*, LSD};
    use abstract_core::{
        adapter::{self},
        objects::{AnsAsset, AssetEntry},
        MANAGER,
    };
    use cw_orch::interface;
    use cw_orch::prelude::*;
    use cosmwasm_std::{Decimal, Empty};

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct DexAdapter<Chain>;

    // Implement deployer trait
    impl<Chain: CwEnv> AdapterDeployer<Chain, LsdInstantiateMsg> for DexAdapter<Chain> {}

    impl<Chain: CwEnv> Uploadable for DexAdapter<Chain> {
        fn wrapper(&self) -> <Mock as TxHandler>::ContractSource {
            Box::new(
                ContractWrapper::new_with_empty(
                    crate::contract::execute,
                    crate::contract::instantiate,
                    crate::contract::query,
                )
            )
        }
        fn wasm(&self) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path("abstract_lsd_adapter")
                .unwrap()
        }
    }


    impl<Chain: CwEnv> DexAdapter<Chain>{
        /// Swap using Abstract's OS (registered in daemon_state).
        pub fn swap(
            &self,
            offer_asset: (&str, u128),
            ask_asset: &str,
            lsd: String,
        ) -> Result<(), AbstractInterfaceError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());
            let asset = AssetEntry::new(offer_asset.0);
            let ask_asset = AssetEntry::new(ask_asset);

            let swap_msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: LsdExecuteMsg::Action {
                    lsd,
                    action: LsdAction::Swap {
                        offer_asset: AnsAsset::new(asset, offer_asset.1),
                        ask_asset,
                        max_spread: Some(Decimal::percent(30)),
                        belief_price: None,
                    },
                },
            });
            manager.execute_on_module(LSD, swap_msg)?;
            Ok(())
        }
    }
}
