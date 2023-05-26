
use cosmwasm_std::{Deps};
use cw_asset::Asset;

use super::identity::Identify;

/// # LsdQuery
/// ensures LSD adapters support the expected functionality.
///
/// Implements the usual LSD queries. These queries are mainly about what assets are used withing the LSD mechanism
pub trait LsdQuery: Identify {
    /// The underlying asset that is staked and auto-compounded by the LSD
    fn underlying_token(&self, deps: Deps) -> Result<Asset>;

    /// The LSD token that auto-compounds staked assets rewards
    fn lsd_token(&self, deps: Deps) -> Result<Asset>;
}
