//! # Decentralized Exchange Adapter
//!
//! [`abstract_dex_adapter`] is a generic dex-interfacing contract that handles address retrievals and dex-interactions.

use cw_asset::AssetBase;
use cosmwasm_std::Addr;
use abstract_core::{
    adapter,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{CosmosMsg, Uint128};

pub type LsdName = String;

// TODO, what's this ?
pub const IBC_DEX_ID: u32 = 11335;

pub type ExecuteMsg = adapter::ExecuteMsg<LsdExecuteMsg>;
pub type QueryMsg = adapter::QueryMsg<LsdQueryMsg>;
pub type InstantiateMsg = adapter::InstantiateMsg<LsdInstantiateMsg>;

impl adapter::AdapterExecuteMsg for LsdExecuteMsg {}
impl adapter::AdapterQueryMsg for LsdQueryMsg {}

#[cosmwasm_schema::cw_serde]
pub struct LsdInstantiateMsg { }

/// Dex Execute msg
#[cosmwasm_schema::cw_serde]
pub enum LsdExecuteMsg {
    Action {
        lsd: LsdName,
        action: LsdAction,
    },
}

/// Possible actions to perform on the LSD token (or hub)
#[cosmwasm_schema::cw_serde]
pub enum LsdAction {
    /// Bond your native tokens and get LSD tokens in return
    Bond {
        // amount of tokens to bond
        amount: Uint128,
        // The address of the asset is already inputted into the adapter
    },
    /// Unbond your LSD tokens and get native tokens in return when the burn period is over
    Unbond{
        // amount of tokens to bond
        amount: Uint128
        // The address of the asset is already inputted into the adapter
    },
    /// Claim all the unlocked native tokens once the burn period is over
    Claim {
        // We claim all rewards at the same time
    }
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "cw-orch", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "cw-orch", impl_into(QueryMsg))]
pub enum LsdQueryMsg {
    /// Gets the underlying token of the LSD
    // TODO, hopefully our 2 messages have the same return type, this is not really possible if they don't have the same return type ?
    #[returns(InfoResponse)]
    Info{
        lsd: LsdName,
        query: LsdInfo
    },
    /// Endpoint can be used by front-end to easily interact with contracts.
    #[returns(GenerateMessagesResponse)]
    GenerateMessages { message: LsdExecuteMsg },
}

/// Possible queries to perform on the LSD token (or hub)
#[cosmwasm_schema::cw_serde]
pub enum LsdInfo {
    /// Bond your native tokens and get LSD tokens in return
    UnderlyingToken{},
    /// Unbond your LSD tokens and get native tokens in return when the burn period is over
    LSDToken{},
}

// Response from QueryMsg::Info
#[cosmwasm_schema::cw_serde]
pub enum InfoResponse{
    UnderlyingToken(AssetBase<Addr>),
    LSDToken(AssetBase<Addr>),
}

/// Response from GenerateMsgs
#[cosmwasm_schema::cw_serde]
pub struct GenerateMessagesResponse {
    /// messages generated for dex action
    pub messages: Vec<CosmosMsg>,
}
