
use cosmwasm_std::Deps;
use cosmwasm_std::Uint128;
use cosmwasm_std::CosmosMsg;
use crate::error::LsdError;
use crate::traits::command::LsdCommand;
use crate::traits::identity::Identify;
use crate::traits::query::LsdQuery;

pub const ERIS_PROTOCOL: &str = "eris_protocol";

pub struct ErisProtocol {}

impl Identify for ErisProtocol {
    fn name(&self) -> &'static str {
        ERIS_PROTOCOL
    }
    fn over_ibc(&self) -> bool {
        false
    }
}

impl LsdQuery for ErisProtocol{

}


impl LsdCommand for ErisProtocol {

}