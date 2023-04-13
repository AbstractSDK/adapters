use cosmwasm_std::Addr;
use cw_utils::Duration;
use osmosis_std::shim::Duration as OsmosisDuration;

use crate::{error::StakingError, traits::identify::Identify};

use super::ProviderName;

pub const OSMOSIS: &str = "osmosis";

#[derive(Default)]
pub struct Osmosis {
    pub local_proxy_addr: Option<Addr>,
    pub pool_id: Option<u64>,
    pub pool_denom: Option<String>,
    pub validator_addr: Option<Addr>,
    pub is_superfluid: Option<bool>,
}

impl Identify for Osmosis {
    fn name(&self) -> ProviderName {
        OSMOSIS
    }
}

#[cfg(feature = "osmosis")]
pub mod fns {
    use std::str::FromStr;

    use crate::msg::Claim;
    use crate::{error::StakingError, msg::StakingInfoResponse, CwStakingAdapter};

    use super::*;
    use cosmwasm_std::{CosmosMsg, Deps, StdError, StdResult, Timestamp};
    use cosmwasm_std::{Decimal, Env, Uint128};
    use cw_asset::{AssetInfo, AssetInfoBase};
    use cw_utils::{Duration, Expiration};
    use osmosis_std::types::osmosis::lockup::{AccountLockedDurationRequest, PeriodLock};
    use osmosis_std::types::osmosis::poolincentives::v1beta1::QueryGaugeIdsRequest;
    use osmosis_std::types::osmosis::superfluid::QueryUnpoolWhitelistRequest;
    use osmosis_std::{
        shim::Duration as OsmosisDuration,
        types::cosmos::base::v1beta1::Coin as OsmoCoin,
        types::osmosis::incentives::ActiveGaugesPerDenomRequest,
        types::osmosis::lockup::{MsgBeginUnlocking, MsgLockTokens},
        types::osmosis::superfluid::{
            MsgLockAndSuperfluidDelegate, MsgSuperfluidUndelegateAndUnbondLock,
        },
    };

    /// Osmosis app-chain dex implementation
    impl CwStakingAdapter for Osmosis {
        fn fetch_data(
            &mut self,
            deps: cosmwasm_std::Deps,
            _env: Env,
            ans_host: &abstract_sdk::feature_objects::AnsHost,
            staking_asset: abstract_core::objects::AssetEntry,
        ) -> abstract_sdk::AbstractSdkResult<()> {
            let provider_staking_contract_entry = self.staking_entry(&staking_asset);

            // This is going to be the gamm pool address
            let gamm_pool_address =
                ans_host.query_contract(&deps.querier, &provider_staking_contract_entry)?;

            // This is going to be the gamm pool id
            let gamm_pool_id = gamm_pool_address
                .to_string()
                .split('/')
                .last()
                .unwrap()
                .parse::<u64>()
                .map_err(|_| StdError::generic_err("Unable to parse gamm pool id"))?;

            self.local_proxy_addr = Some(Addr::unchecked(
                "This is going to be the local proxy address",
            ));

            // TODO: this is the validator address
            self.validator_addr = Some(Addr::unchecked(
                "osmovaloper1v9w0j3x7q5yqy0y3q3x6y2z7xqz3zq5q9zq3zq",
            ));

            self.pool_denom = Some(gamm_pool_address.to_string());
            self.pool_id = Some(gamm_pool_id);
            self.is_superfluid =
                Some(self.query_is_superfluid(&deps.querier, self.pool_denom.clone().unwrap())?);

            Ok(())
        }
        /// In the case of *non-superfluid pools*, the flow will be as follows:
        /// -- first time locking tokens --
        /// 1. lock tokens: *MsgLockTokens*
        ///     parameters:
        ///     * duration
        ///     * sender
        ///     * coins
        ///
        /// -- subsequent staking --
        /// 2nd time lock-> lock tokens: *MsgLockTokens*
        ///    parameters: same
        ///
        /// -- unstaking --
        /// 1. unlock tokens: *MsgBeginUnlocking* ( begin-unlock-period-lock)
        ///    parameters:
        ///   * lock_id
        ///   * sender
        ///   * coins  
        ///
        /// ## *superfluid pools*
        /// In the case of superfluid pools, the flow will be as follows:
        /// -- first time locking tokens --
        /// 1st time lock -> lock tokens: *MsgLockAndSuperfluidDelegate*
        ///    parameters:
        ///     * sender
        ///     * coins
        ///     * val_addr
        ///     
        /// -- subsequent staking --
        /// 2nd time lock -> lock tokens: *MsgLockTokens*
        ///   parameters:
        ///     * duration
        ///     * sender
        ///     * coins
        ///
        /// -- unstaking (v15) --
        /// -> unlock tokens: *MsgSuperfluidUndelegateAndUnbondLock* (v15)
        ///   parameters:
        ///    * lock_id
        ///    * sender
        ///    * coins
        ///
        /// -- unstaking (v14) --
        /// -> unlock tokens (two Steps)
        ///     1). *MsgSuperfluidUndelegate*
        ///     parameters:
        ///     * lock_id
        ///     * sender
        ///
        ///    2). *MsgSuperFluidUnbondLock*
        ///   parameters:
        ///   * lock_id
        ///   * sender
        ///
        ///
        fn stake(
            &self,
            deps: Deps,
            amount: cosmwasm_std::Uint128,
            unbonding_period: Option<cw_utils::Duration>,
        ) -> Result<Vec<cosmwasm_std::CosmosMsg>, StakingError> {
            let proxy_addr = self.local_proxy_addr.as_ref().unwrap();
            let duration = unwrap_unbond(self, unbonding_period)?;
            let is_superfluid = self.is_superfluid.unwrap();

            let coin = OsmoCoin {
                denom: self.pool_denom.as_ref().unwrap().to_string(),
                amount: amount.to_string(),
            };

            let lock_id = self.query_lock_id(
                &deps.querier,
                duration.clone(),
                self.local_proxy_addr.as_ref().unwrap(),
            )?;

            let msg: CosmosMsg = match (lock_id, is_superfluid) {
                // No lock id but superfluid => first time superfluid staking
                (None, true) => MsgLockAndSuperfluidDelegate {
                    sender: proxy_addr.to_string(),
                    coins: vec![coin],
                    val_addr: self.validator_addr.as_ref().unwrap().to_string(),
                }
                .into(),
                // all other cases [(lock_id, is_superfluid) => (Some, true), (Some, false), (None, false))]
                _ => MsgLockTokens {
                    duration: Some(duration),
                    owner: self.local_proxy_addr.as_ref().unwrap().to_string(),
                    coins: vec![coin],
                }
                .into(),
            };

            Ok(vec![msg])
        }

        fn unstake(
            &self,
            deps: Deps,
            amount: cosmwasm_std::Uint128,
            unbonding_period: Option<cw_utils::Duration>,
        ) -> Result<Vec<cosmwasm_std::CosmosMsg>, StakingError> {
            let duration = unwrap_unbond(self, unbonding_period)?;
            let proxy_addr = self.local_proxy_addr.as_ref().unwrap();
            let is_superfluid = self.is_superfluid.unwrap();

            let lock_id = self
                .query_lock_id(&deps.querier, duration, proxy_addr)?
                .unwrap();

            let coin = OsmoCoin {
                denom: self.pool_denom.as_ref().unwrap().to_string(),
                amount: amount.to_string(),
            };

            let msg: CosmosMsg = match is_superfluid {
                true => MsgSuperfluidUndelegateAndUnbondLock {
                    sender: proxy_addr.to_string(),
                    lock_id,
                    coin: Some(coin),
                }
                .into(),
                false => MsgBeginUnlocking {
                    id: lock_id,
                    owner: proxy_addr.to_string(),
                    coins: vec![coin],
                }
                .into(),
            };

            Ok(vec![msg])
        }

        fn claim_rewards(&self, _deps: Deps) -> Result<Vec<cosmwasm_std::CosmosMsg>, StakingError> {
            // REWARDS ARE CLAIMED AUTOMATICALLY
            Ok(vec![])
        }

        fn claim(&self, _deps: Deps) -> Result<Vec<cosmwasm_std::CosmosMsg>, StakingError> {
            // UNBONDING TOKENS ARE CLAIMED AUTOMATICALLY
            Ok(vec![])
        }

        fn query_info(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
        ) -> crate::contract::CwStakingResult<crate::msg::StakingInfoResponse> {
            let lockable_durations = QueryGaugeIdsRequest {
                pool_id: self.pool_id.unwrap(),
            }
            .query(querier)
            .unwrap()
            .gauge_ids_with_duration
            .into_iter()
            // TODO: This is a hack to filter out gauges with 0% incentive
            .filter(|g| Decimal::from_str(&g.gauge_incentive_percentage).unwrap() > Decimal::zero())
            .map(|g| Duration::Time(g.duration.unwrap().seconds as u64))
            .collect::<Vec<_>>();

            let res = StakingInfoResponse {
                staking_token: AssetInfoBase::Cw20(Addr::unchecked(
                    self.pool_denom.as_ref().unwrap().to_string(),
                )),
                staking_contract_address: Addr::unchecked(self.pool_denom.as_ref().unwrap()),
                unbonding_periods: Some(lockable_durations),
                max_claims: None,
            };

            Ok(res)
        }

        fn query_staked(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
            staker: Addr,
            unbonding_period: Option<cw_utils::Duration>,
        ) -> crate::contract::CwStakingResult<crate::msg::StakeResponse> {
            let duration = unwrap_unbond(self, unbonding_period)?;
            let lock = self.query_lock(querier, duration, &staker, Some(false))?;
            if let Some(lock) = lock {
                Ok(crate::msg::StakeResponse {
                    amount: lock.coins.first().unwrap().amount.parse().unwrap(),
                })
            } else {
                Ok(crate::msg::StakeResponse {
                    amount: Uint128::zero(),
                })
            }
        }

        fn query_unbonding(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
            staker: Addr,
        ) -> crate::contract::CwStakingResult<crate::msg::UnbondingResponse> {
            let unbonding = unwrap_unbond(self, Some(Duration::Time(0)))?;
            let locks = self.query_locks(querier, unbonding, &staker, Some(true))?;
            let claims = locks
                .into_iter()
                .map(|lock| Claim {
                    amount: lock.coins.first().unwrap().amount.parse().unwrap(),
                    claimable_at: Expiration::AtTime(
                        Timestamp::from_seconds(lock.end_time.as_ref().unwrap().seconds as u64)
                            .plus_nanos(lock.end_time.unwrap().nanos as u64),
                    ),
                })
                .collect::<Vec<Claim>>();

            let response = crate::msg::UnbondingResponse { claims };
            Ok(response)
        }

        fn query_reward_tokens(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
        ) -> crate::contract::CwStakingResult<crate::msg::RewardTokensResponse> {
            // NOTE: This query is super inefficient but i dont know how to do it better
            let reward_tokens = ActiveGaugesPerDenomRequest {
                denom: self.pool_denom.as_ref().unwrap().to_string(),
                pagination: None,
            }
            .query(querier)
            .unwrap()
            .data
            .into_iter()
            .filter(|gauge| {
                if gauge.is_perpetual {
                    true
                } else {
                    gauge.num_epochs_paid_over > gauge.filled_epochs
                }
            })
            .flat_map(|g| g.coins)
            .map(|coin| AssetInfo::Native(coin.denom))
            .collect::<Vec<_>>();

            Ok(crate::msg::RewardTokensResponse {
                tokens: reward_tokens,
            })
        }
    }

    impl Osmosis {
        /// queries the lock_id for the given duration and account(staker)
        fn query_lock_id(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
            duration: OsmosisDuration,
            staker: &Addr,
        ) -> Result<Option<u64>, StakingError> {
            let lock = self.query_lock(querier, duration, staker, Some(false))?;

            match lock {
                Some(lock) => Ok(Some(lock.id)),
                None => Ok(None),
            }
        }

        /// Query the lock with the exact duration.
        fn query_lock(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
            duration: OsmosisDuration,
            staker: &Addr,
            is_unbonding: Option<bool>,
        ) -> StdResult<Option<PeriodLock>> {
            let locks = self.query_locks(querier, duration, staker, is_unbonding)?;
            // so we have to filter out the synthetic locks
            let locks = locks
                .into_iter()
                .filter(|p| p.end_time.is_none())
                .collect::<Vec<_>>();
            Ok(locks.first().cloned())
        }

        /// Query all the locks that are equal or longer than the duration and
        /// they are sorted ascending by duration.
        fn query_locks(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
            duration: OsmosisDuration,
            staker: &Addr,
            is_unbonding: Option<bool>,
        ) -> Result<Vec<PeriodLock>, StdError> {
            // query all the locks for the given staker and duration and then
            // filter out the ones that are not for the pool denom
            // this query returns the lock with the current staked, but also all the synthetic locks,
            // which are locks that are currently unbonding.
            let mut locks = AccountLockedDurationRequest {
                duration: Some(duration),
                owner: staker.to_string(),
            }
            .query(querier)?
            .locks;

            if let Some(denom) = self.pool_denom.as_ref() {
                locks = locks
                    .into_iter()
                    .filter(|p| p.coins.first().unwrap().denom.eq(denom))
                    .collect::<Vec<_>>();
            }

            if let Some(is_unbonding) = is_unbonding {
                locks = locks
                    .into_iter()
                    .filter(|p| p.end_time.is_some() == is_unbonding)
                    .collect::<Vec<_>>();
            }

            Ok(locks)
        }

        /// queries wether the given denom is a superfluid asset or not
        fn query_is_superfluid(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
            denom: String,
        ) -> StdResult<bool> {
            let res = QueryUnpoolWhitelistRequest {}.query(querier)?.pool_ids;
            if res.contains(
                &denom
                    .split('/')
                    .last()
                    .unwrap()
                    .parse::<u64>()
                    .map_err(|_| StdError::generic_err("Unable to parse gamm pool id"))?,
            ) {
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}

fn unwrap_unbond(
    dex: &Osmosis,
    unbonding_period: Option<Duration>,
) -> Result<OsmosisDuration, StakingError> {
    let Some(Duration::Time(unbonding_period)) = unbonding_period else {
        if unbonding_period.is_none() {
            return Err(StakingError::UnbondingPeriodNotSet(dex.name().to_owned()));
        } else {
            return Err(StakingError::UnbondingPeriodNotSupported("height".to_owned(), dex.name().to_owned()));
        }
    };
    Ok(OsmosisDuration {
        seconds: unbonding_period as i64,
        nanos: 0,
    })
}
