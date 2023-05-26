use abstract_adapter::AdapterError;

use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum LsdError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    AbstractOs(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    AdapterError(#[from] AdapterError),

    #[error("LSD {0} is not a known lsd on this network.")]
    UnknownLSD(String),

    #[error("LSD {0} is not local to this network.")]
    ForeignLSD(String),

    #[error("Asset type: {0} is unsupported.")]
    UnsupportedAssetType(String),

    #[error("Can't bond with no amount")]
    BondAmountZero,

    #[error("Can't unbond with no amount")]
    UnbondAmountZero,

    #[error("Not implemented for lsd {0}")]
    NotImplemented(String),

    #[error("Message generation for IBC queries not supported.")]
    IbcMsgQuery,
    
    #[error("Invalid Generate Message")]
    InvalidGenerateMessage,
}
