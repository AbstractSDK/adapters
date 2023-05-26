
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
    Ok(Response::default())
}
