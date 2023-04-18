pub mod contract;
pub mod error;
pub mod msg;

pub const TOKEN_FACTORY: &str = "abstract:token-factory";

#[cfg(feature = "boot")]
pub mod boot {
    use abstract_boot::boot_core::ContractWrapper;
    use abstract_boot::boot_core::{contract, Contract, CwEnv};
    use abstract_boot::ApiDeployer;
    use abstract_core::api::InstantiateMsg;
    use cosmwasm_std::Empty;

    use crate::msg::*;

    #[contract(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct TokenFactoryApi<Chain>;

    impl<Chain: CwEnv> ApiDeployer<Chain, Empty> for TokenFactoryApi<Chain> {}

    impl<Chain: CwEnv> TokenFactoryApi<Chain> {
        pub fn new(name: &str, chain: Chain) -> Self {
            Self(
                Contract::new(name, chain)
                    .with_wasm_path("abstract_token_factory_api")
                    .with_mock(Box::new(ContractWrapper::new_with_empty(
                        crate::contract::execute,
                        crate::contract::instantiate,
                        crate::contract::query,
                    ))),
            )
        }
    }
}
