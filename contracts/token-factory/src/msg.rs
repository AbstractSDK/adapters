//! # Tendermint Staking Api
//!
//! `abstract_core::token_factory` exposes all the function of [`cosmwasm_std::CosmosMsg::Staking`] and [`cosmwasm_std::CosmosMsg::Distribution`].

use crate::error::TokenFactoryError;
use abstract_core::api;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{DepsMut, StdResult, Uint128};
use token_bindings::{
    AdminResponse, DenomsByCreatorResponse, FullDenomResponse, Metadata as SubdenomMetadata,
    MetadataResponse, ParamsResponse, TokenFactoryQuery, TokenMsg, TokenQuerier, TokenQuery,
};

pub type ExecuteMsg = api::ExecuteMsg<TokenFactoryExecuteMsg>;
pub type QueryMsg = api::QueryMsg<TokenFactoryQueryMsg>;

impl api::ApiExecuteMsg for TokenFactoryExecuteMsg {}
impl api::ApiQueryMsg for TokenFactoryQueryMsg {}

/// Token Factory API execute messages
/// See [`token_bindings::TokenMsg`] for more details.
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
#[cfg_attr(feature = "boot", impl_into(ExecuteMsg))]
pub enum TokenFactoryExecuteMsg {
    /// CreateDenom creates a new factory denom, of denomination:
    /// factory/{creating contract bech32 address}/{Subdenom}
    /// Subdenom can be of length at most 44 characters, in [0-9a-zA-Z./]
    /// Empty subdenoms are valid.
    /// The (creating contract address, subdenom) pair must be unique.
    /// The created denom's admin is the creating contract address,
    /// but this admin can be changed using the UpdateAdmin binding.
    ///
    /// If you set an initial metadata here, this is equivalent
    /// to calling SetMetadata directly on the returned denom.
    CreateDenom {
        subdenom: String,
        metadata: Option<SubdenomMetadata>,
    },
    /// ChangeAdmin changes the admin for a factory denom.
    /// Can only be called by the current contract admin.
    /// If the NewAdminAddress is empty, the denom will have no admin.
    ChangeAdmin {
        denom: String,
        new_admin_address: String,
    },
    /// Contracts can mint native tokens for an existing factory denom
    /// that they are the admin of.
    MintTokens {
        denom: String,
        amount: Uint128,
        mint_to_address: String,
    },
    /// Contracts can burn native tokens for an existing factory denom
    /// that they are the admin of.
    BurnTokens {
        denom: String,
        amount: Uint128,
        burn_from_address: String,
    },
    /// Contracts can force transfer tokens for an existing factory denom
    /// that they are the admin of.
    ForceTransfer {
        denom: String,
        amount: Uint128,
        from_address: String,
        to_address: String,
    },
    SetMetadata {
        denom: String,
        metadata: SubdenomMetadata,
    },
}

impl TokenFactoryExecuteMsg {
    pub fn validate(&self, deps: DepsMut<TokenFactoryQuery>) -> Result<(), TokenFactoryError> {
        match self {
            TokenFactoryExecuteMsg::CreateDenom { subdenom, .. } => {
                if subdenom.eq("") {
                    return Err(TokenFactoryError::InvalidSubdenom {
                        subdenom: subdenom.to_string(),
                    });
                }
            }
            TokenFactoryExecuteMsg::ChangeAdmin {
                new_admin_address,
                denom,
            } => {
                deps.api.addr_validate(&new_admin_address)?;

                validate_denom(deps, denom.clone())?;
            }
            TokenFactoryExecuteMsg::MintTokens {
                denom,
                mint_to_address,
                amount,
            } => {
                deps.api.addr_validate(&mint_to_address)?;

                if amount.eq(&Uint128::zero()) {
                    return Err(TokenFactoryError::ZeroAmount {});
                }

                validate_denom(deps, denom.clone())?;
            }
            TokenFactoryExecuteMsg::BurnTokens {
                denom,
                burn_from_address,
                amount,
            } => {
                if !burn_from_address.is_empty() {
                    return Err(TokenFactoryError::BurnFromAddressNotSupported {
                        address: burn_from_address.to_string(),
                    });
                }

                if amount.eq(&Uint128::new(0_u128)) {
                    return Err(TokenFactoryError::ZeroAmount {});
                }

                validate_denom(deps, denom.clone())?;
            }
            _ => unimplemented!(),
        }
        Ok(())
    }
}

impl From<TokenMsg> for TokenFactoryExecuteMsg {
    fn from(msg: TokenMsg) -> Self {
        match msg {
            TokenMsg::CreateDenom { subdenom, metadata } => {
                TokenFactoryExecuteMsg::CreateDenom { subdenom, metadata }
            }
            TokenMsg::ChangeAdmin {
                denom,
                new_admin_address,
            } => TokenFactoryExecuteMsg::ChangeAdmin {
                denom,
                new_admin_address,
            },
            TokenMsg::MintTokens {
                denom,
                amount,
                mint_to_address,
            } => TokenFactoryExecuteMsg::MintTokens {
                denom,
                amount,
                mint_to_address,
            },
            TokenMsg::BurnTokens {
                denom,
                amount,
                burn_from_address,
            } => TokenFactoryExecuteMsg::BurnTokens {
                denom,
                amount,
                burn_from_address,
            },
            TokenMsg::ForceTransfer {
                denom,
                amount,
                from_address,
                to_address,
            } => TokenFactoryExecuteMsg::ForceTransfer {
                denom,
                amount,
                from_address,
                to_address,
            },
            TokenMsg::SetMetadata { denom, metadata } => {
                TokenFactoryExecuteMsg::SetMetadata { denom, metadata }
            }
        }
    }
}

fn validate_denom(
    deps: DepsMut<TokenFactoryQuery>,
    denom: String,
) -> Result<(), TokenFactoryError> {
    let denom_to_split = denom.clone();
    let tokenfactory_denom_parts: Vec<&str> = denom_to_split.split('/').collect();

    if tokenfactory_denom_parts.len() != 3 {
        return Err(TokenFactoryError::InvalidDenom {
            denom,
            message: std::format!(
                "denom must have 3 parts separated by /, had {}",
                tokenfactory_denom_parts.len()
            ),
        });
    }

    let prefix = tokenfactory_denom_parts[0];
    let creator_address = tokenfactory_denom_parts[1];
    let subdenom = tokenfactory_denom_parts[2];

    if !prefix.eq_ignore_ascii_case("factory") {
        return Err(TokenFactoryError::InvalidDenom {
            denom,
            message: std::format!("prefix must be 'factory', was {}", prefix),
        });
    }

    // Validate denom by attempting to query for full denom
    let response = TokenQuerier::new(&deps.querier)
        .full_denom(String::from(creator_address), String::from(subdenom));
    if response.is_err() {
        return Err(TokenFactoryError::InvalidDenom {
            denom,
            message: response.err().unwrap().to_string(),
        });
    }

    Ok(())
}

/// Token Factory API query messages
/// TODO: include something to do with ANS?
/// See [`token_bindings::TokenQuery`] for more details.
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
#[cfg_attr(feature = "boot", impl_into(QueryMsg))]
pub enum TokenFactoryQueryMsg {
    /// Given a subdenom created by the address `creator_addr` via `OsmosisMsg::CreateDenom`,
    /// returns the full denom as used by `BankMsg::Send`.
    /// You may call `FullDenom { creator_addr: env.contract.address, subdenom }` to find the denom issued
    /// by the current contract.
    /// Returns [`FullDenomResponse`]
    #[returns(FullDenomResponse)]
    FullDenom {
        creator_addr: String,
        subdenom: String,
    },
    /// Returns the metadata set for this denom, if present. May return None.
    /// This will also return metadata for native tokens created outside
    /// of the token factory (like staking tokens)
    /// Returns [`MetadataResponse`]
    #[returns(MetadataResponse)]
    Metadata { denom: String },
    /// Returns info on admin of the denom, only if created/managed via token factory.
    /// Errors if denom doesn't exist or was created by another module.
    /// Returns [`AdminResponse`]
    #[returns(AdminResponse)]
    Admin { denom: String },
    /// List all denoms that were created by the given creator.
    /// This does not imply all tokens currently managed by the creator.
    /// (Admin may have changed)
    /// Returns [`DenomsByCreatorResponse`]
    #[returns(DenomsByCreatorResponse)]
    DenomsByCreator { creator: String },
    /// Returns configuration params for TokenFactory modules
    /// Returns [`ParamsResponse`]
    #[returns(ParamsResponse)]
    Params {},
}

impl From<TokenFactoryQueryMsg> for TokenQuery {
    fn from(msg: TokenFactoryQueryMsg) -> Self {
        match msg {
            TokenFactoryQueryMsg::FullDenom {
                creator_addr,
                subdenom,
            } => TokenQuery::FullDenom {
                creator_addr,
                subdenom,
            },
            TokenFactoryQueryMsg::Metadata { denom } => TokenQuery::Metadata { denom },
            TokenFactoryQueryMsg::Admin { denom } => TokenQuery::Admin { denom },
            TokenFactoryQueryMsg::DenomsByCreator { creator } => {
                TokenQuery::DenomsByCreator { creator }
            }
            TokenFactoryQueryMsg::Params {} => TokenQuery::Params {},
        }
    }
}
