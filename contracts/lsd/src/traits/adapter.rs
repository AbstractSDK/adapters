use crate::msg::LsdInfo;
use crate::error::LsdError;
use crate::msg::{LsdAction};

use abstract_sdk::features::AbstractNameService;
use abstract_sdk::Execution;
use cosmwasm_std::{CosmosMsg, Deps, Uint128};

use super::command::LsdCommand;
use super::query::LsdQuery;

// TODO CHANGE AND FIND INDICES ?
pub const BOND: u64 = 7542;
pub const UNBOND: u64 = 7543;
pub const CLAIM: u64 = 7546;

impl<T> LsdAdapter for T where T: AbstractNameService + Execution {}

pub(crate) type ReplyId = u64;

pub trait LsdAdapter: AbstractNameService + Execution {
    /// resolve the provided LSD action on a local LSD
    fn resolve_lsd_action(
        &self,
        deps: Deps,
        action: LsdAction,
        lsd: &dyn LsdCommand,
    ) -> Result<(Vec<CosmosMsg>, ReplyId), LsdError> {
        Ok(match action {
            LsdAction::Bond {
                amount
            } => {
                if amount.is_zero() {
                    return Err(LsdError::BondAmountZero);
                }
                (
                    self.resolve_bond(deps, lsd, amount)?,
                    BOND,
                )
            }
            LsdAction::Unbond {
                amount
            } => {
                if amount.is_zero() {
                    return Err(LsdError::UnbondAmountZero);
                }
                (
                    self.resolve_unbond(
                        deps,
                        lsd, amount
                    )?,
                    UNBOND,
                )
            }
            LsdAction::Claim { } => (
                self.resolve_claim(deps, lsd)?,
                CLAIM,
            ),
        })
    }

    fn resolve_bond(
        &self,
        deps: Deps,
        lsd: &dyn LsdCommand,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg>, LsdError> {
        let bond_msgs = lsd.bond(deps, amount)?;
        Ok(bond_msgs)
    }

    fn resolve_unbond(
        &self,
        deps: Deps,
        lsd: &dyn LsdCommand,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg>, LsdError> {
        let unbond_msgs = lsd.unbond(deps, amount)?;
        Ok(unbond_msgs)
    }

    fn resolve_claim(
        &self,
        deps: Deps,
        lsd: &dyn LsdCommand,
    ) -> Result<Vec<CosmosMsg>, LsdError> {
        let claim_msgs = lsd.claim(deps)?;
        Ok(claim_msgs)
    }
}
