use crate::error::LsdError;
use crate::lsds::{lsd_resolver};
use crate::msg::{LsdAction, LsdExecuteMsg, LsdName, IBC_DEX_ID};
use crate::{
    contract::{LsdAdapter, LsdResult},
};
use abstract_core::ibc_client::CallbackInfo;
use abstract_core::objects::ans_host::AnsHost;
use abstract_core::objects::AnsAsset;
use abstract_sdk::{features::AbstractNameService, Execution};
use abstract_sdk::{IbcInterface, Resolve};
use cosmwasm_std::{to_binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError};

const ACTION_RETRIES: u8 = 3;

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    adapter: LsdAdapter,
    msg: LsdExecuteMsg,
) -> LsdResult {
    match msg {
        LsdExecuteMsg::Action {
            lsd: lsd_name,
            action,
        } => {
            let exchange = lsd_resolver::identify_lsd(&lsd_name)?;
            // if exchange is on an app-chain, execute the action on the app-chain
            if exchange.over_ibc() {
                handle_ibc_request(&deps, info, &adapter, lsd_name, &action)
            } else {
                // the action can be executed on the local chain
                handle_local_request(deps, env, info, adapter, action, lsd_name)
            }
        }
    }
}

/// Handle an adapter request that can be executed on the local chain
fn handle_local_request(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    adapter: LsdAdapter,
    action: LsdAction,
    lsd_name: LsdName,
) -> LsdResult {
    let lsd = lsd_resolver::resolve_lsd(&lsd_name)?;
    let (msgs, _) = crate::traits::adapter::LsdAdapter::resolve_lsd_action(
        &adapter,
        deps.as_ref(),
        action,
        lsd,
    )?;
    let proxy_msg = adapter.executor(deps.as_ref()).execute(msgs)?;
    Ok(Response::new().add_message(proxy_msg))
}

/// Handle an adapter request that can be executed on an IBC chain
fn handle_ibc_request(
    deps: &DepsMut,
    info: MessageInfo,
    adapter: &LsdAdapter,
    dex_name: LsdName,
    action: &LsdAction,
) -> LsdResult {
    let host_chain = dex_name;
    let ans = adapter.name_service(deps.as_ref());
    let ibc_client = adapter.ibc_client(deps.as_ref());
    // get the to-be-sent assets from the action
    let coins = resolve_assets_to_transfer(deps.as_ref(), action, ans.host())?;
    // construct the ics20 call(s)
    let ics20_transfer_msg = ibc_client.ics20_transfer(host_chain.clone(), coins)?;
    // construct the action to be called on the host
    let action = abstract_sdk::core::ibc_host::HostAction::App {
        msg: to_binary(&action)?,
    };
    let maybe_contract_info = deps.querier.query_wasm_contract_info(info.sender.clone());
    let callback = if maybe_contract_info.is_err() {
        None
    } else {
        Some(CallbackInfo {
            id: IBC_DEX_ID.to_string(),
            receiver: info.sender.into_string(),
        })
    };
    let ibc_action_msg = ibc_client.host_action(host_chain, action, callback, ACTION_RETRIES)?;

    // call both messages on the proxy
    Ok(Response::new().add_messages(vec![ics20_transfer_msg, ibc_action_msg]))
}

pub(crate) fn resolve_assets_to_transfer(
    deps: Deps,
    dex_action: &LsdAction,
    ans_host: &AnsHost,
) -> LsdResult<Vec<Coin>> {
    // resolve asset to native asset
    let offer_to_coin = |offer: &AnsAsset| {
        offer
            .resolve(&deps.querier, ans_host)?
            .try_into()
            .map_err(LsdError::from)
    };

    match dex_action {
        LsdAction::ProvideLiquidity { assets, .. } => {
            let coins: Result<Vec<Coin>, _> = assets.iter().map(offer_to_coin).collect();
            coins
        }
        LsdAction::ProvideLiquiditySymmetric { .. } => Err(LsdError::Std(StdError::generic_err(
            "Cross-chain symmetric provide liquidity not supported.",
        ))),
        LsdAction::WithdrawLiquidity { lp_token, amount } => Ok(vec![offer_to_coin(&AnsAsset {
            name: lp_token.to_owned(),
            amount: amount.to_owned(),
        })?]),
        LsdAction::Swap { offer_asset, .. } => Ok(vec![offer_to_coin(offer_asset)?]),
        LsdAction::CustomSwap { offer_assets, .. } => {
            let coins: Result<Vec<Coin>, _> = offer_assets.iter().map(offer_to_coin).collect();
            coins
        }
    }
    .map_err(Into::into)
}
