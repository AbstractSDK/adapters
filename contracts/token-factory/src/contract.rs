use crate::msg::{TokenFactoryExecuteMsg, TokenFactoryQueryMsg};
use crate::{error::TokenFactoryError, factory, factory2, TOKEN_FACTORY};
use abstract_api::ApiContract;
use abstract_sdk::Execution;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use token_bindings::{TokenFactoryMsg, TokenFactoryQuery, TokenMsg, TokenQuerier, TokenQuery};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type TokenFactoryApi =
    ApiContract<TokenFactoryError, Empty, TokenFactoryExecuteMsg, TokenFactoryQueryMsg>;

pub type TokenFactoryResult<T = CosmosMsg<TokenFactoryMsg>> = Result<T, TokenFactoryError>;

const STAKING_API: TokenFactoryApi = TokenFactoryApi::new(TOKEN_FACTORY, CONTRACT_VERSION, None)
    .with_execute(execute_handler)
    .with_query(query_handler);

// Export handlers
#[cfg(feature = "export")]
abstract_api::export_endpoints!(STAKING_API, TokenFactoryApi);

// TODO: add token to ans when created
pub fn execute_handler(
    deps: DepsMut<TokenFactoryQuery>,
    _env: Env,
    _info: MessageInfo,
    api: TokenFactoryApi,
    msg: TokenFactoryExecuteMsg,
) -> TokenFactoryResult<Response> {
    msg.validate(deps)?;
    let token_msg: TokenMsg = msg.into();
    let executor = api.executor(deps.as_ref());

    let action = match token_msg {
        TokenMsg::CreateDenom { .. } => "create_denom",
        TokenMsg::MintTokens { .. } => "mint_tokens",
        TokenMsg::BurnTokens { .. } => "burn_tokens",
        TokenMsg::ChangeAdmin { .. } => "change_admin",
        TokenMsg::ForceTransfer { .. } => "force_transfer",
        TokenMsg::SetMetadata { .. } => "set_metadata",
    };

    Ok(executor.execute_with_response(vec![token_msg.into()], action)?)
}

pub fn query_handler(
    deps: Deps<TokenFactoryQuery>,
    _env: Env,
    api: TokenFactoryApi,
    msg: TokenFactoryQueryMsg,
) -> StdResult<Binary> {
    let token_query: TokenQuery = msg.into();
    deps.querier.query(&token_query.into())
}
