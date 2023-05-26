use crate::error::LsdError;
use crate::traits::query::LsdQuery;
use crate::traits::{command::LsdCommand, identity::Identify};



// Supported lsds for now
pub use crate::lsds::eris::{ERIS_PROTOCOL, ErisProtocol};

pub(crate) fn identify_lsd(value: &str) -> Result<&'static dyn Identify, LsdError> {
    match value {
        ERIS_PROTOCOL => Ok(&ErisProtocol {}),
        _ => Err(LsdError::UnknownLSD(value.to_owned())),
    }
}

pub(crate) fn resolve_lsd(value: &str) -> Result<&'static dyn LsdCommand, LsdError> {
    match value {
        ERIS_PROTOCOL => Ok(&ErisProtocol {}),
        _ => Err(LsdError::ForeignLSD(value.to_owned())),
    }
}

pub(crate) fn resolve_lsd_query(value: &str) -> Result<&'static dyn LsdQuery, LsdError> {
    match value {
        ERIS_PROTOCOL => Ok(&ErisProtocol {}),
        _ => Err(LsdError::ForeignLSD(value.to_owned())),
    }
}
