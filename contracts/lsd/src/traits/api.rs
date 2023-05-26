// TODO: this should be moved to the public dex package
// It cannot be in abstract-os because it does not have a dependency on sdk (as it shouldn't)
use cw_asset::AssetBase;
use cosmwasm_std::Addr;
use crate::{
    msg::{
        LsdAction, LsdExecuteMsg, LsdQueryMsg, LsdName, LsdInfo, InfoResponse,
    },
    LSD,
};
use abstract_core::objects::{module::ModuleId};
use abstract_sdk::{AdapterInterface, AbstractSdkError};
use abstract_sdk::{
    features::{AccountIdentification, Dependencies},
    AbstractSdkResult,
};
use cosmwasm_std::{CosmosMsg, Deps, Uint128};
use serde::de::DeserializeOwned;

// API for Abstract SDK users
/// Interact with the dex adapter in your module.
pub trait LsdInterface: AccountIdentification + Dependencies {
    /// Construct a new dex interface
    fn dex<'a>(&'a self, deps: Deps<'a>, name: LsdName) -> Lsd<Self> {
        Lsd {
            base: self,
            deps,
            name,
            module_id: LSD,
        }
    }
}

impl<T: AccountIdentification + Dependencies> LsdInterface for T {}

#[derive(Clone)]
pub struct Lsd<'a, T: LsdInterface> {
    base: &'a T,
    name: LsdName,
    module_id: ModuleId<'a>,
    deps: Deps<'a>,
}

impl<'a, T: LsdInterface> Lsd<'a, T> {
    /// Set the module id for the LSD
    pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
        Self { module_id, ..self }
    }

    /// returns DEX name
    fn lsd_name(&self) -> LsdName {
        self.name.clone()
    }

    /// returns the DEX module id
    fn lsd_module_id(&self) -> ModuleId {
        self.module_id
    }

    /// Executes a [LSDAction] in the LSD
    fn request(&self, action: LsdAction) -> AbstractSdkResult<CosmosMsg> {
        let adapters = self.base.adapters(self.deps);

        adapters.request(
            self.lsd_module_id(),
            LsdExecuteMsg::Action {
                lsd: self.lsd_name(),
                action,
            },
        )
    }

    /// Bond assets in the LSD
    pub fn bond(
        &self,
        amount: Uint128,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(LsdAction::Bond {
            amount
        })
    }

    /// Unbond assets from the LSD
    pub fn unbond(
        &self,
        amount: Uint128,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(LsdAction::Unbond {
            amount
        })
    }


    /// Claim the unbounded tokens
    pub fn claim(
        &self,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(LsdAction::Claim {  })
    }
}

impl<'a, T: LsdInterface> Lsd<'a, T> {
    /// Do a query in the LSD
    fn query<R: DeserializeOwned>(&self, query_msg: LsdQueryMsg) -> AbstractSdkResult<R> {
        let adapters = self.base.adapters(self.deps);
        adapters.query(LSD, query_msg)
    }

    /// Do a query in the LSD query adapter
    fn lsd_query(&self, query_msg: LsdInfo) -> AbstractSdkResult<InfoResponse> {
        let query_result: InfoResponse = self.query(LsdQueryMsg::Info { lsd: self.lsd_module_id().to_owned(), query: query_msg })?;
        Ok(query_result)
    }

    pub fn underlying_token(&self) -> AbstractSdkResult<AssetBase<Addr>>{
        let response = self.lsd_query(LsdInfo::UnderlyingToken {  })?;
        match response{
            InfoResponse::UnderlyingToken(i) => Ok(i),
            _ => Err(AbstractSdkError::generic_err("Couldn't unwrap query result"))
        }
    }

    pub fn lsd_token(&self) -> AbstractSdkResult<AssetBase<Addr>>{
        let response = self.lsd_query(LsdInfo::LSDToken {  })?; 
        match response{
            InfoResponse::LSDToken(i) => Ok(i),
            _ => Err(AbstractSdkError::generic_err("Couldn't unwrap query result"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::msg::ExecuteMsg;
    use abstract_core::adapter::AdapterRequestMsg;
    use abstract_sdk::mock_module::MockModule;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::wasm_execute;
    use speculoos::prelude::*;

    fn expected_request_with_test_proxy(request: DexExecuteMsg) -> ExecuteMsg {
        AdapterRequestMsg {
            proxy_address: Some(abstract_testing::prelude::TEST_PROXY.to_string()),
            request,
        }
        .into()
    }

    #[test]
    fn swap_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex = stub
            .dex(deps.as_ref(), "junoswap".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let dex_name = "junoswap".to_string();
        let offer_asset = OfferAsset::new("juno", 1000u128);
        let ask_asset = AssetEntry::new("uusd");
        let max_spread = Some(Decimal::percent(1));
        let belief_price = Some(Decimal::percent(2));

        let expected = expected_request_with_test_proxy(DexExecuteMsg::Action {
            dex: dex_name,
            action: DexAction::Swap {
                offer_asset: offer_asset.clone(),
                ask_asset: ask_asset.clone(),
                max_spread,
                belief_price,
            },
        });

        let actual = dex.swap(offer_asset, ask_asset, max_spread, belief_price);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn custom_swap_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex_name = "astroport".to_string();

        let dex = stub
            .dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let offer_assets = vec![OfferAsset::new("juno", 1000u128)];
        let ask_assets = vec![AskAsset::new("uusd", 1000u128)];
        let max_spread = Some(Decimal::percent(1));
        let router = Some(SwapRouter::Custom("custom_router".to_string()));

        let expected = expected_request_with_test_proxy(DexExecuteMsg::Action {
            dex: dex_name,
            action: DexAction::CustomSwap {
                offer_assets: offer_assets.clone(),
                ask_assets: ask_assets.clone(),
                max_spread,
                router: router.clone(),
            },
        });

        let actual = dex.custom_swap(offer_assets, ask_assets, max_spread, router);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn provide_liquidity_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex_name = "junoswap".to_string();

        let dex = stub
            .dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let assets = vec![OfferAsset::new("taco", 1000u128)];
        let max_spread = Some(Decimal::percent(1));

        let expected = expected_request_with_test_proxy(DexExecuteMsg::Action {
            dex: dex_name,
            action: DexAction::ProvideLiquidity {
                assets: assets.clone(),
                max_spread,
            },
        });

        let actual = dex.provide_liquidity(assets, max_spread);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn provide_liquidity_symmetric_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex_name = "junoswap".to_string();

        let dex = stub
            .dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let offer = OfferAsset::new("taco", 1000u128);
        let paired = vec![AssetEntry::new("bell")];
        let _max_spread = Some(Decimal::percent(1));

        let expected = expected_request_with_test_proxy(DexExecuteMsg::Action {
            dex: dex_name,
            action: DexAction::ProvideLiquiditySymmetric {
                offer_asset: offer.clone(),
                paired_assets: paired.clone(),
            },
        });

        let actual = dex.provide_liquidity_symmetric(offer, paired);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn withdraw_liquidity_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex_name = "junoswap".to_string();

        let dex = stub
            .dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let lp_token = AssetEntry::new("taco");
        let withdraw_amount: Uint128 = 1000u128.into();

        let expected = expected_request_with_test_proxy(DexExecuteMsg::Action {
            dex: dex_name,
            action: DexAction::WithdrawLiquidity {
                lp_token: lp_token.clone(),
                amount: withdraw_amount,
            },
        });

        let actual = dex.withdraw_liquidity(lp_token, withdraw_amount);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }
}
