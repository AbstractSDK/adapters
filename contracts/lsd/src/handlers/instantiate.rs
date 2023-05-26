use crate::contract::{LsdAdapter, LsdResult};
use crate::{msg::LsdInstantiateMsg};

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    adapter: LsdAdapter,
    msg: LsdInstantiateMsg,
) -> LsdResult {
    let recipient = adapter
        .account_registry(deps.as_ref())
        .proxy_address(msg.recipient_account)?;
    Ok(Response::default())
}
