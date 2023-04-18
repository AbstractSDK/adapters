use crate::contract::CwStakingResult;
use crate::error::StakingError;
use crate::msg::{RewardTokensResponse, StakeResponse, StakingInfoResponse, UnbondingResponse};
use crate::traits::identify::Identify;
use abstract_core::objects::LpToken;
use abstract_sdk::core::objects::{AssetEntry, ContractEntry};
use abstract_sdk::feature_objects::AnsHost;
use abstract_sdk::AbstractSdkResult;
use cosmwasm_std::{Addr, CosmosMsg, Deps, Env, QuerierWrapper, StdResult, Uint128};

use cw_utils::Duration;

/// Trait that defines the adapter interface for staking providers
pub trait CwStakingAdapter: Identify {
    /// Construct a staking contract entry from the staking token and the provider
    fn staking_entry(&self, staking_token: &AssetEntry) -> ContractEntry {
        ContractEntry {
            protocol: self.name().to_string(),
            contract: format!("staking/{staking_token}"),
        }
    }
    /// Retrieve the staking contract address for the pool with the provided staking token name
    fn staking_contract_address(
        &self,
        deps: Deps,
        ans_host: &AnsHost,
        staking_token: &AssetEntry,
    ) -> AbstractSdkResult<Addr> {
        let provider_staking_contract_entry = self.staking_entry(staking_token);
        ans_host
            .query_contract(&deps.querier, &provider_staking_contract_entry)
            .map_err(Into::into)
    }

    /// Build the LP token from the provided asset entry
    fn provider_lp_token(&self, staking_asset: &AssetEntry) -> StdResult<LpToken> {
        let lp_token = AssetEntry::new(&format!("{}/{}", self.name(), staking_asset));
        LpToken::try_from(lp_token)
    }

    /// Fetch the required data for interacting with the provider
    fn fetch_data(
        &mut self,
        deps: Deps,
        env: Env,
        ans_host: &AnsHost,
        staking_asset: AssetEntry,
    ) -> AbstractSdkResult<()>;

    /// Stake the provided asset into the staking contract
    ///
    /// * `deps` - the dependencies
    /// * `asset` - the asset to stake
    /// * `unbonding_period` - the unbonding period to use for the stake
    fn stake(
        &self,
        deps: Deps,
        amount: Uint128,
        unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, StakingError>;

    /// Stake the provided asset into the staking contract
    ///
    /// * `deps` - the dependencies
    /// * `asset` - the asset to stake
    /// * `unbonding_period` - the unbonding period to use for the unstake
    fn unstake(
        &self,
        deps: Deps,
        amount: Uint128,
        unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, StakingError>;

    /// Claim rewards on the staking contract
    ///
    /// * `deps` - the dependencies
    fn claim_rewards(&self, deps: Deps) -> Result<Vec<CosmosMsg>, StakingError>;

    /// Claim matured unbonding claims on the staking contract
    fn claim(&self, deps: Deps) -> Result<Vec<CosmosMsg>, StakingError>;

    fn query_info(&self, querier: &QuerierWrapper) -> CwStakingResult<StakingInfoResponse>;
    // This function queries the staked token balance of a staker
    // The staking contract is queried using the staking address
    fn query_staked(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
        unbonding_period: Option<Duration>,
    ) -> CwStakingResult<StakeResponse>;
    fn query_unbonding(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
    ) -> CwStakingResult<UnbondingResponse>;
    fn query_reward_tokens(
        &self,
        querier: &QuerierWrapper,
    ) -> CwStakingResult<RewardTokensResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::identify::Identify;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    const MOCK_PROVIDER_NAME: &str = "mock";

    struct MockProvider;

    impl Identify for MockProvider {
        fn name(&self) -> &'static str {
            MOCK_PROVIDER_NAME
        }
    }

    impl CwStakingAdapter for MockProvider {
        fn fetch_data(
            &mut self,
            _deps: Deps,
            _env: Env,
            _ans_host: &AnsHost,
            _staking_asset: AssetEntry,
        ) -> AbstractSdkResult<()> {
            unimplemented!()
        }

        fn stake(
            &self,
            _deps: Deps,
            _amount: Uint128,
            _unbonding_period: Option<Duration>,
        ) -> Result<Vec<CosmosMsg>, StakingError> {
            unimplemented!()
        }

        fn unstake(
            &self,
            _deps: Deps,
            _amount: Uint128,
            _unbonding_period: Option<Duration>,
        ) -> Result<Vec<CosmosMsg>, StakingError> {
            unimplemented!()
        }

        fn claim_rewards(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, StakingError> {
            unimplemented!()
        }

        fn claim(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, StakingError> {
            unimplemented!()
        }

        fn query_info(&self, _querier: &QuerierWrapper) -> CwStakingResult<StakingInfoResponse> {
            unimplemented!()
        }

        fn query_staked(
            &self,
            _querier: &QuerierWrapper,
            _staker: Addr,
            _unbonding_period: Option<Duration>,
        ) -> CwStakingResult<StakeResponse> {
            unimplemented!()
        }

        fn query_unbonding(
            &self,
            _querier: &QuerierWrapper,
            _staker: Addr,
        ) -> CwStakingResult<UnbondingResponse> {
            unimplemented!()
        }

        fn query_reward_tokens(
            &self,
            _querier: &QuerierWrapper,
        ) -> CwStakingResult<RewardTokensResponse> {
            unimplemented!()
        }
    }

    const TEST_STAKING_ASSET: &str = "terra2>astro,terra2>luna";

    mod staking_entry {
        use super::*;

        #[test]
        fn it_builds_the_staking_contract_entry() {
            let provider = MockProvider;
            let staking_token = AssetEntry::new(TEST_STAKING_ASSET);
            let expected = ContractEntry {
                protocol: MOCK_PROVIDER_NAME.to_string(),
                contract: "staking/terra2>astro,terra2>luna".to_string(),
            };

            let actual = provider.staking_entry(&staking_token);

            assert_that!(actual).is_equal_to(expected);
        }
    }

    mod provider_lp_token {
        use super::*;

        #[test]
        fn it_returns_the_lp_token() {
            let provider = MockProvider;
            let staking_token = AssetEntry::new(TEST_STAKING_ASSET);
            let expected =
                LpToken::try_from(AssetEntry::new("mock/terra2>astro,terra2>luna")).unwrap();

            let actual = provider.provider_lp_token(&staking_token).unwrap();

            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn err_when_invalid_asset_entry() {
            let provider = MockProvider;
            let staking_token = AssetEntry::new("invalid");

            let actual = provider.provider_lp_token(&staking_token);

            assert_that!(actual)
                .is_err()
                .matches(|e| matches!(e, cosmwasm_std::StdError::GenericErr { .. }));
        }
    }

    mod staking_contract_address {
        use abstract_testing::prelude::{AbstractMockQuerierBuilder, TEST_ANS_HOST};

        use super::*;
        use abstract_core::ans_host::state::CONTRACT_ADDRESSES;

        #[test]
        fn it_returns_the_staking_contract_address() {
            let mut deps = mock_dependencies();
            let staking_contract_entry = ContractEntry {
                protocol: MOCK_PROVIDER_NAME.to_string(),
                contract: format!("staking/{}", TEST_STAKING_ASSET),
            };
            let staking_contract_address = Addr::unchecked("staking_contract_address");

            // Mock the contract entry in ANS
            deps.querier = AbstractMockQuerierBuilder::default()
                .builder()
                .with_contract_map_entries(
                    TEST_ANS_HOST,
                    CONTRACT_ADDRESSES,
                    vec![(&staking_contract_entry, staking_contract_address.clone())],
                )
                .build();

            let ans_host = AnsHost::new(Addr::unchecked(TEST_ANS_HOST));

            let provider = MockProvider;

            let actual = provider
                .staking_contract_address(
                    deps.as_ref(),
                    &ans_host,
                    &AssetEntry::new(TEST_STAKING_ASSET),
                )
                .unwrap();

            assert_that!(actual).is_equal_to(staking_contract_address);
        }
    }
}
