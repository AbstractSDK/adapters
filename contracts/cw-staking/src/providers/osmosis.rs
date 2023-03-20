use cosmwasm_std::Addr;
use cw_utils::Duration;
use osmosis_std::{
    shim::Duration as OsmosisDuration, types::osmosis::superfluid::SuperfluidAssetType,
};

use crate::{error::StakingError, traits::identify::Identify};

pub const OSMOSIS: &str = "osmosis";

#[derive(Default)]
pub struct Osmosis {
    pub local_proxy_addr: Option<Addr>,
    pub pool_id: Option<u64>,
    pub pool_denom: Option<String>,
    pub lock_id: Option<u64>,
    pub validator_addr: Option<Addr>,
    pub is_superfluid: Option<bool>,
}

impl Identify for Osmosis {
    fn name(&self) -> &'static str {
        OSMOSIS
    }
}

#[cfg(feature = "osmosis")]
pub mod fns {
    use crate::msg::Claim;
    use crate::{error::StakingError, msg::StakingInfoResponse, CwStakingAdapter};

    use super::*;
    const FORTEEN_DAYS: i64 = 60 * 60 * 24 * 14;
    use cosmwasm_std::{CosmosMsg, Deps, Querier, StdError, StdResult, Timestamp, Uint256};
    use cosmwasm_std::{Decimal, Decimal256, Env, Uint128};
    use cw_asset::{Asset, AssetInfoBase};
    use cw_utils::{Duration, Expiration};
    use osmosis_std::types::osmosis::lockup::{
        AccountLockedLongerDurationDenomRequest, AccountUnlockingCoinsRequest, PeriodLock,
    };
    use osmosis_std::types::osmosis::superfluid::AssetTypeRequest;
    use osmosis_std::{
        shim::Duration as OsmosisDuration,
        types::osmosis::gamm::v1beta1::{
            MsgJoinPool, MsgSwapExactAmountIn, QuerySwapExactAmountInRequest,
        },
        types::osmosis::{
            gamm::v1beta1::QueryPoolResponse,
            superfluid::{MsgLockAndSuperfluidDelegate, MsgSuperfluidUndelegateAndUnbondLock},
        },
        types::{
            cosmos::base::query,
            osmosis::lockup::{MsgBeginUnlocking, MsgLockTokens},
        },
        types::{
            cosmos::base::v1beta1::Coin as OsmoCoin,
            osmosis::gamm::v1beta1::{Pool, QueryPoolRequest},
        },
    };

    /// Osmosis app-chain dex implementation
    impl CwStakingAdapter for Osmosis {
        /// Fetching data for osmosis staking involves:
        /// 1. Fetch the pool address from ANS (has pool id)
        /// 2. Fetch the lock_id from osmosis (can be done with pool address, address and duration)
        /// 3.
        fn fetch_data(
            &mut self,
            deps: cosmwasm_std::Deps,
            env: Env,
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
                .split("/")
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

            // TODO: this is the superfluid lock id
            self.pool_denom = Some(gamm_pool_address.to_string());
            self.pool_id = Some(gamm_pool_id);

            // superfluid asset type 0 is for non-superfluid pools and 1 is for superfluid pools
            // see docs: https://github.com/osmosis-labs/osmosis/tree/main/x/superfluid#assettype
            self.is_superfluid =
                Some(self.query_is_superfluid(&deps.querier, self.pool_denom.clone().unwrap())?);

            Ok(())
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
                // NOTE: This shold be the gamm token address
                denom: self.pool_denom.as_ref().unwrap().to_string(),
                amount: amount.to_string(),
            };

            let lock_id = self.query_lock_id(
                duration.clone(),
                self.local_proxy_addr.as_ref().unwrap(),
                &deps.querier,
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

        // TODO: workout edgecase of not superfluid staking pools
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
                .query_lock_id(duration, proxy_addr, &deps.querier)?
                .unwrap();

            let coin = OsmoCoin {
                // NOTE: This shold be the gamm token address ??
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

        fn claim(&self, _deps: Deps) -> Result<Vec<cosmwasm_std::CosmosMsg>, StakingError> {
            // Check: Im not sure if this is correct
            // Claim is not nesseccary for osmosis
            Ok(vec![])
        }

        // fn query_pool_data(
        //     &self,
        //     querier: &cosmwasm_std::QuerierWrapper,
        //     pool_id: u64,
        // ) -> StdResult<Pool> {
        //     let res = QueryPoolRequest { pool_id }.query(&querier).unwrap();

        //     let pool = Pool::try_from(res.pool.unwrap()).unwrap();
        //     Ok(pool)
        // }

        fn query_info(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
        ) -> crate::contract::CwStakingResult<crate::msg::StakingInfoResponse> {
            let pool = self
                .query_pool_data(querier, self.pool_id.unwrap())
                .unwrap();

            let res = StakingInfoResponse {
                staking_token: AssetInfoBase::Cw20(Addr::unchecked(
                    self.pool_denom.as_ref().unwrap().to_string(),
                )),
                staking_contract_address: Addr::unchecked(self.pool_denom.as_ref().unwrap()),
                unbonding_periods: Some(vec![]),
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
            let lock = self.query_lock(duration, &staker, querier)?;

            let amount = match lock {
                Some(lock) => lock.coins.iter().fold(Uint128::zero(), |acc, coin| {
                    acc + coin.amount.parse::<Uint128>().unwrap()
                }),
                None => Uint128::zero(),
            };

            Ok(crate::msg::StakeResponse { amount })
        }

        fn query_unbonding(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
            staker: Addr,
        ) -> crate::contract::CwStakingResult<crate::msg::UnbondingResponse> {
            // NOTE: THIS IS NOT CORRECT -> We dont have unbonding period here, so we have to return the sum of all locks
            let unbonding = unwrap_unbond(self, Some(Duration::Time(0)))?;

            let locks = self.query_locks(unbonding, &staker, querier)?;
            let response = crate::msg::UnbondingResponse {
                claims: locks
                    .into_iter()
                    .map(|lock| Claim {
                        amount: lock.coins.first().unwrap().amount.parse().unwrap(),
                        claimable_at: Expiration::AtTime(
                            Timestamp::from_seconds(lock.end_time.as_ref().unwrap().seconds as u64)
                                .plus_nanos(lock.end_time.unwrap().nanos as u64),
                        ),
                    })
                    .collect::<Vec<Claim>>(),
            };
            Ok(response)
        }

        fn query_reward_tokens(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
        ) -> crate::contract::CwStakingResult<crate::msg::RewardTokensResponse> {
            todo!()
        }

    impl Osmosis {
        fn query_lock_id(
            &self,
            duration: OsmosisDuration,
            owner: &Addr,
            querier: &cosmwasm_std::QuerierWrapper,
        ) -> Result<Option<u64>, StakingError> {
            // This query returns all the locks that are equal or longer than the duration
            // there is no query that returns the lock with the exact duration AND
            // osmosis docs do not specify the order if there are multiple locks
            // so we have to sort based on duration and take the first, which should be the one that equals the duration
            let lock = self.query_lock(duration, owner, querier)?;

            match lock {
                Some(lock) => Ok(Some(lock.id)),
                None => Ok(None),
            }
        }

        fn compute_osmo_share_out_amount(
            pool_assets: &[OsmoCoin],
            deposits: &[Uint128; 2],
            total_share: Uint128,
        ) -> StdResult<Uint128> {
            // ~ source: terraswap contract ~
            // min(1, 2)
            // 1. sqrt(deposit_0 * exchange_rate_0_to_1 * deposit_0) * (total_share / sqrt(pool_0 * pool_1))
            // == deposit_0 * total_share / pool_0
            // 2. sqrt(deposit_1 * exchange_rate_1_to_0 * deposit_1) * (total_share / sqrt(pool_1 * pool_1))
            // == deposit_1 * total_share / pool_1
            let share_amount_out = std::cmp::min(
                deposits[0].multiply_ratio(
                    total_share,
                    pool_assets[0].amount.parse::<Uint128>().unwrap(),
                ),
                deposits[1].multiply_ratio(
                    total_share,
                    pool_assets[1].amount.parse::<Uint128>().unwrap(),
                ),
            );

            Ok(share_amount_out)
        }

        fn assert_slippage_tolerance(
            slippage_tolerance: &Option<Decimal>,
            deposits: &[Uint128; 2],
            pool_assets: &[OsmoCoin],
        ) -> Result<(), StakingError> {
            if let Some(slippage_tolerance) = *slippage_tolerance {
                let slippage_tolerance: Decimal256 = slippage_tolerance.into();
                if slippage_tolerance > Decimal256::one() {
                    return Err(StakingError::Std(StdError::generic_err(
                        "slippage_tolerance cannot bigger than 1",
                    )));
                }

                let one_minus_slippage_tolerance = Decimal256::one() - slippage_tolerance;
                let deposits: [Uint256; 2] = [deposits[0].into(), deposits[1].into()];
                let pools: [Uint256; 2] = [
                    pool_assets[0].amount.parse::<Uint256>().unwrap(),
                    pool_assets[1].amount.parse::<Uint256>().unwrap(),
                ];

                // Ensure each prices are not dropped as much as slippage tolerance rate
                if Decimal256::from_ratio(deposits[0], deposits[1]) * one_minus_slippage_tolerance
                    > Decimal256::from_ratio(pools[0], pools[1])
                    || Decimal256::from_ratio(deposits[1], deposits[0])
                        * one_minus_slippage_tolerance
                        > Decimal256::from_ratio(pools[1], pools[0])
                {
                    return Err(StakingError::MaxSlippageAssertion(
                        slippage_tolerance.to_string(),
                        OSMOSIS.to_owned(),
                    ));
                }
            }

            Ok(())
        }

        fn query_pool_data(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
            pool_id: u64,
        ) -> StdResult<Pool> {
            let res = QueryPoolRequest { pool_id }.query(&querier).unwrap();

            let pool = Pool::try_from(res.pool.unwrap()).unwrap();
            Ok(pool)
        }

        fn query_is_superfluid(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
            denom: String,
        ) -> StdResult<bool> {
            let asset_type = AssetTypeRequest { denom }.query(&querier)?.asset_type;

            match asset_type {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(StdError::generic_err("invalid asset type")),
            }
        }

        fn query_lock(
            &self,
            duration: OsmosisDuration,
            staker: &Addr,
            querier: &cosmwasm_std::QuerierWrapper,
        ) -> StdResult<Option<PeriodLock>> {
            let locks = self.query_locks(duration, staker, querier)?;

            Ok(locks.first().cloned())
        }

        fn query_locks(
            &self,
            duration: OsmosisDuration,
            staker: &Addr,
            querier: &cosmwasm_std::QuerierWrapper,
        ) -> Result<Vec<PeriodLock>, StdError> {
            let mut locks = AccountLockedLongerDurationDenomRequest {
                duration: Some(duration),
                denom: self.pool_denom.as_ref().unwrap().to_string(),
                owner: staker.to_string(),
            }
            .query(&querier)?
            .locks;
            locks.sort_by(|a, b| {
                a.duration
                    .as_ref()
                    .unwrap()
                    .seconds
                    .cmp(&b.duration.as_ref().unwrap().seconds)
            });
            Ok(locks)
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
