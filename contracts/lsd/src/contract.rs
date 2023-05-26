use crate::msg::{LsdExecuteMsg, LsdInstantiateMsg, LsdQueryMsg};
use crate::LSD;
use crate::{error::LsdError, handlers};
use abstract_adapter::{export_endpoints, AdapterContract};
use cosmwasm_std::Response;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type LsdAdapter = AdapterContract<LsdError, LsdInstantiateMsg, LsdExecuteMsg, LsdQueryMsg>;
pub type LsdResult<T = Response> = Result<T, LsdError>;

pub const LSD_ADAPTER: LsdAdapter = LsdAdapter::new(LSD, CONTRACT_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler);

#[cfg(feature = "export")]
export_endpoints!(LSD_ADAPTER, LsdAdapter);
