pub mod contract;
pub mod error;
mod lsds;
pub(crate) mod handlers;
pub mod msg;
mod traits;
pub(crate) mod util;

// Export interface for use in SDK modules
pub use traits::api::{Lsd, LsdInterface};

pub const LSD: &str = "abstract:lsd";

#[cfg(feature = "cw-orch")]
pub mod cw_orch {
    use abstract_interface::AbstractInterfaceError;
use abstract_interface::Manager;
    use abstract_interface::AdapterDeployer;
    use cosmwasm_std::Uint128;
    use crate::{msg::*, LSD};
    use abstract_core::{
        adapter::{self},
        MANAGER,
    };
    use cw_orch::interface;
    use cw_orch::prelude::*;
    use cosmwasm_std::{Empty};

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
        pub fn bond(
            &self,
            lsd: String,
            amount: Uint128,
        ) -> Result<(), AbstractInterfaceError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());

            let bond_msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: LsdExecuteMsg::Action {
                    lsd,
                    action: LsdAction::Bond {
                        amount
                    }
                },
            });
            manager.execute_on_module(LSD, bond_msg)?;
            Ok(())
        }
    }

    impl<Chain: CwEnv> DexAdapter<Chain>{
        /// Swap using Abstract's OS (registered in daemon_state).
        pub fn unbond(
            &self,
            lsd: String,
            amount: Uint128,
        ) -> Result<(), AbstractInterfaceError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());

            let bond_msg = crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: LsdExecuteMsg::Action {
                    lsd,
                    action: LsdAction::Unbond {
                        amount
                    }
                },
            });
            manager.execute_on_module(LSD, bond_msg)?;
            Ok(())
        }
    }
}
