use crate::error::LsdError;
use cosmwasm_std::{CosmosMsg, Deps, Uint128};

use super::identity::Identify;

pub type Return = Uint128;
pub type Spread = Uint128;
pub type Fee = Uint128;
pub type FeeOnInput = bool;

/// # LsdCommand
/// ensures LSD adapters support the expected functionality.
///
/// Implements the usual LSD operations.
pub trait LsdCommand: Identify {
    /// Bonds native assets on the protocol using the protocol in question custom logic
    fn bond(
        &self,
        deps: Deps,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg>, LsdError>;

    /// Bonds native assets on the protocol using the protocol in question custom logic
    fn unbond(
        &self,
        deps: Deps,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg>, LsdError>;

    /// Bonds native assets on the protocol using the protocol in question custom logic
    fn claim(
        &self,
        deps: Deps,
    ) -> Result<Vec<CosmosMsg>, LsdError>;
}
