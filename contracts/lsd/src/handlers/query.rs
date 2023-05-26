use crate::msg::{LsdInfo, InfoResponse};


use crate::msg::{
    LsdExecuteMsg, LsdQueryMsg, GenerateMessagesResponse,
};
use crate::{
    contract::{LsdAdapter, LsdResult},
    error::LsdError,
    lsds::lsd_resolver,
};

use cosmwasm_std::{to_binary, Binary, Deps, Env};

pub fn query_handler(
    deps: Deps,
    env: Env,
    adapter: &LsdAdapter,
    msg: LsdQueryMsg,
) -> LsdResult<Binary> {
    match msg {
        LsdQueryMsg::Info {
            lsd: lsd_name,
            query
        } => {     
            handle_local_query(deps, env, adapter, query, lsd_name)
        },
        LsdQueryMsg::GenerateMessages { message } => {
            match message {
                LsdExecuteMsg::Action { lsd: lsd_name, action } => {
                    let lsd_id = lsd_resolver::identify_lsd(&lsd_name)?;
                    // if exchange is on an app-chain, execute the action on the app-chain
                    if lsd_id.over_ibc() {
                        return Err(LsdError::IbcMsgQuery);
                    }

                    let lsd = lsd_resolver::resolve_lsd(&lsd_name)?;
                    let (messages, _) = crate::traits::adapter::LsdAdapter::resolve_lsd_action(
                        adapter, deps, action, lsd,
                    )?;
                    to_binary(&GenerateMessagesResponse { messages }).map_err(Into::into)
                }
                _ => Err(LsdError::InvalidGenerateMessage {}),
            }
        }
    }
}


/// Handle an adapter request that can be executed on the local chain
fn handle_local_query(
    deps: Deps,
    env: Env,
    adapter: &LsdAdapter,
    query: LsdInfo,
    lsd: String,
) -> LsdResult<Binary> {
    let lsd = lsd_resolver::resolve_lsd_query(&lsd)?;
    let query_result = match query{
        LsdInfo::UnderlyingToken {  } => {
            let result = lsd.underlying_token(deps)?;
            to_binary(&InfoResponse::UnderlyingToken(result))?
        },
        LsdInfo::LSDToken {  } => {
            let result = lsd.lsd_token(deps)?;
            to_binary(&InfoResponse::LSDToken(result))?
        },
    };
    Ok(query_result)
}
