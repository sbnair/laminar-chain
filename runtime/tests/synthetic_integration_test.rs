/// tests for this module

#[cfg(test)]

mod tests {
	use frame_support::{assert_noop, assert_ok};
	use laminar_runtime::{
		tests::*,
		CurrencyId::{AUSD, FEUR, FJPY},
		Runtime,
	};
	use orml_prices::Price;
	use sp_runtime::Permill;

	#[test]
	fn test_synthetic_buy_and_sell() {
		ExtBuilder::default()
			.balances(vec![
				(POOL::get(), AUSD, dollar(10_000)),
				(ALICE::get(), AUSD, dollar(10_000)),
			])
			.build()
			.execute_with(|| {
				assert_ok!(synthetic_create_pool());
				assert_ok!(synthetic_set_enabled_trades());
				assert_ok!(synthetic_deposit_liquidity(&POOL::get(), dollar(10_000)));
				assert_ok!(synthetic_set_min_additional_collateral_ratio(Permill::from_percent(10)));
				assert_ok!(synthetic_set_additional_collateral_ratio(
					FEUR,
					Permill::from_percent(10)
				));
				assert_ok!(synthetic_set_spread(FEUR, Permill::from_percent(1)));
				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(3, 1))]));

				assert_eq!(collateral_balance(&ALICE::get()), dollar(10_000));
				assert_eq!(collateral_balance(&POOL::get()), 0);
				assert_eq!(synthetic_liquidity(), dollar(10_000));
				assert_eq!(synthetic_balance(), 0);
				assert_ok!(synthetic_buy(&ALICE::get(), FEUR, dollar(5000)));
				assert_eq!(collateral_balance(&ALICE::get()), dollar(5000));
				// synthetic = collateral / ask_price
				// 1650 ≈ 5000 / (3 * (1 + 0.01))
				//assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), dollar(1650));
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 1650165016501650165016);
				// additional_collateral = (synthetic * price) * (1 + ratio) - collateral
				// 445 = (1650 * 3.0) * (1 + 0.1) - 5000
				// 5000 = ALICE -> ModuleTokens
				// 445 = LiquidityPool -> ModuleTokens
				//assert_eq!(synthetic_balance(), dollar(5445));
				assert_eq!(synthetic_balance(), 5445544554455445544553);
				// collateralise = balance - additional_collateral
				// 9555 = 10_000 - 445
				//assert_eq!(liquidity(), dollar(9555));
				assert_eq!(synthetic_liquidity(), 9554455445544554455447);

				assert_ok!(synthetic_sell(&ALICE::get(), FEUR, dollar(800)));
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 850165016501650165016);
				// collateral = synthetic * bid_price
				// 2376 = 800 * (3 * (1 - 0.01))
				assert_eq!(collateral_balance(&ALICE::get()), dollar(7376));
				// redeem_collateral = collateral_position - (synthetic * price) * (1 + ratio)
				// 2805 = (850 * 3) * (1 + 0.1)
				assert_eq!(synthetic_balance(), 2805544554455445544553);
				// 2376 = ModuleTokens -> ALICE
				// 264 = 5445 - 2805 - 2376
				// 264 = ModuleTokens -> LiquidityPool
				// 9819 = 9555 + 264
				assert_eq!(synthetic_liquidity(), 9818455445544554455447);
			});
	}

	#[test]
	fn test_synthetic_buy_all_of_collateral() {
		ExtBuilder::default()
			.balances(vec![(POOL::get(), AUSD, 1000), (ALICE::get(), AUSD, 1000)])
			.build()
			.execute_with(|| {
				assert_ok!(synthetic_create_pool());
				assert_ok!(synthetic_set_enabled_trades());
				assert_ok!(synthetic_deposit_liquidity(&POOL::get(), 1000));
				assert_ok!(synthetic_set_additional_collateral_ratio(
					FEUR,
					Permill::from_percent(100)
				));
				assert_ok!(synthetic_set_spread(FEUR, Permill::from_percent(1)));
				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(1, 1))]));

				assert_eq!(collateral_balance(&ALICE::get()), 1000);
				assert_eq!(collateral_balance(&POOL::get()), 0);
				assert_eq!(synthetic_liquidity(), 1000);
				assert_eq!(synthetic_balance(), 0);
				assert_ok!(synthetic_buy(&ALICE::get(), FEUR, 1000));
				// synthetic = collateral / ask_price
				// 990 ≈ 1000 / (1 * (1 + 0.01))
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 990);
				// balance = balance - (synthetic * ask_price)
				// 0 ≈ 1000 - (990 * 1.01)
				assert_eq!(collateral_balance(&ALICE::get()), 0);
				// additional_collateral = (synthetic * price) * (1 + ratio) - collateral
				// 980  = (990 * 1.0) * (1 + 1) - 1000
				// 1000 = ALICE -> ModuleTokens
				// 980 = LiquidityPool -> ModuleTokens
				assert_eq!(synthetic_balance(), 1980);
				// collateralise = balance - additional_collateral
				// 20 = 1000 - 980
				assert_eq!(synthetic_liquidity(), 20);

				assert_ok!(synthetic_sell(&ALICE::get(), FEUR, 990));
				// synthetic balance is 190, below ExistentialDeposit
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 0);
				// collateral = synthetic * bid_price
				// 980 = 990 * (1 * (1 - 0.01))
				assert_eq!(collateral_balance(&ALICE::get()), 980);
				// redeem_collateral = collateral_position - (synthetic * price) * (1 + ratio)
				// 0 = (0 * 1) * (1 + 0.1)
				assert_eq!(synthetic_balance(), 0);
				// 980 = ModuleTokens -> ALICE
				// 1000 = 1980 - 980
				// 1000 = ModuleTokens -> LiquidityPool
				// 1020 = 1000 + 20
				assert_eq!(synthetic_liquidity(), 1020);
			});
	}

	#[test]
	fn test_synthetic_trader_take_profit() {
		ExtBuilder::default()
			.balances(vec![
				(POOL::get(), AUSD, dollar(10_000)),
				(ALICE::get(), AUSD, dollar(10_000)),
			])
			.build()
			.execute_with(|| {
				assert_ok!(synthetic_create_pool());
				assert_ok!(synthetic_set_enabled_trades());
				assert_ok!(synthetic_deposit_liquidity(&POOL::get(), dollar(10_000)));
				assert_ok!(synthetic_set_min_additional_collateral_ratio(Permill::from_percent(10)));
				assert_ok!(synthetic_set_additional_collateral_ratio(
					FEUR,
					Permill::from_percent(10)
				));
				assert_ok!(synthetic_set_spread(FEUR, Permill::from_percent(1)));
				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(3, 1))]));

				assert_eq!(collateral_balance(&ALICE::get()), dollar(10_000));
				assert_eq!(collateral_balance(&POOL::get()), 0);
				assert_eq!(synthetic_liquidity(), dollar(10_000));
				assert_eq!(synthetic_balance(), 0);
				assert_ok!(synthetic_buy(&ALICE::get(), FEUR, dollar(5000)));
				assert_eq!(collateral_balance(&ALICE::get()), dollar(5000));
				//assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), dollar(1650));
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 1650165016501650165016);
				//assert_eq!(synthetic_balance(), dollar(5445));
				assert_eq!(synthetic_balance(), 5445544554455445544553);
				//assert_eq!(synthetic_liquidity(), dollar(9555));
				assert_eq!(synthetic_liquidity(), 9554455445544554455447);

				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(31, 10))]));

				assert_ok!(synthetic_sell(
					&ALICE::get(),
					FEUR,
					multi_currency_balance(&ALICE::get(), FEUR)
				));
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 0);
				assert_eq!(collateral_balance(&ALICE::get()), 10064356435643564356434);
				assert_eq!(synthetic_balance(), 0);
				assert_eq!(synthetic_liquidity(), 9935643564356435643566);
			});
	}

	#[test]
	fn test_synthetic_trader_stop_lost() {
		ExtBuilder::default()
			.balances(vec![
				(POOL::get(), AUSD, dollar(10_000)),
				(ALICE::get(), AUSD, dollar(10_000)),
			])
			.build()
			.execute_with(|| {
				assert_ok!(synthetic_create_pool());
				assert_ok!(synthetic_set_enabled_trades());
				assert_ok!(synthetic_deposit_liquidity(&POOL::get(), dollar(10_000)));
				assert_ok!(synthetic_set_min_additional_collateral_ratio(Permill::from_percent(10)));
				assert_ok!(synthetic_set_additional_collateral_ratio(
					FEUR,
					Permill::from_percent(10)
				));
				assert_ok!(synthetic_set_spread(FEUR, Permill::from_percent(1)));
				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(3, 1))]));

				assert_eq!(collateral_balance(&ALICE::get()), dollar(10_000));
				assert_eq!(collateral_balance(&POOL::get()), 0);
				assert_eq!(synthetic_liquidity(), dollar(10_000));
				assert_eq!(synthetic_balance(), 0);
				assert_ok!(synthetic_buy(&ALICE::get(), FEUR, dollar(5000)));
				assert_eq!(collateral_balance(&ALICE::get()), dollar(5000));
				//assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), dollar(1650));
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 1650165016501650165016);
				//assert_eq!(synthetic_balance(), dollar(5445));
				assert_eq!(synthetic_balance(), 5445544554455445544553);
				//assert_eq!(synthetic_liquidity(), dollar(9555));
				assert_eq!(synthetic_liquidity(), 9554455445544554455447);

				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(2, 1))]));

				assert_ok!(synthetic_sell(
					&ALICE::get(),
					FEUR,
					multi_currency_balance(&ALICE::get(), FEUR)
				));
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 0);
				assert_eq!(collateral_balance(&ALICE::get()), 8267326732673267326731);
				assert_eq!(synthetic_balance(), 0);
				assert_eq!(synthetic_liquidity(), 11732673267326732673269);
			});
	}

	#[test]
	fn test_synthetic_multiple_users() {
		ExtBuilder::default()
			.balances(vec![
				(POOL::get(), AUSD, dollar(20_000)),
				(ALICE::get(), AUSD, dollar(10_000)),
				(BOB::get(), AUSD, dollar(10_000)),
			])
			.build()
			.execute_with(|| {
				assert_ok!(synthetic_create_pool());
				assert_ok!(synthetic_set_enabled_trades());
				assert_ok!(synthetic_deposit_liquidity(&POOL::get(), dollar(20_000)));
				assert_ok!(synthetic_set_min_additional_collateral_ratio(Permill::from_percent(10)));
				assert_ok!(synthetic_set_additional_collateral_ratio(
					FEUR,
					Permill::from_percent(10)
				));
				assert_ok!(synthetic_set_spread(FEUR, Permill::from_percent(1)));
				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(3, 1))]));

				assert_eq!(collateral_balance(&POOL::get()), 0);
				assert_eq!(collateral_balance(&ALICE::get()), dollar(10_000));
				assert_eq!(collateral_balance(&BOB::get()), dollar(10_000));
				assert_eq!(synthetic_liquidity(), dollar(20_000));
				assert_eq!(synthetic_balance(), 0);

				// ALICE buy synthetic
				assert_ok!(synthetic_buy(&ALICE::get(), FEUR, dollar(5000)));
				assert_eq!(collateral_balance(&ALICE::get()), dollar(5000));
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 1650165016501650165016);
				assert_eq!(synthetic_balance(), 5445544554455445544553);
				assert_eq!(synthetic_liquidity(), 19554455445544554455447);

				// BOB buy synthetic
				assert_ok!(synthetic_buy(&BOB::get(), FEUR, dollar(5000)));
				assert_eq!(collateral_balance(&BOB::get()), dollar(5000));
				assert_eq!(multi_currency_balance(&BOB::get(), FEUR), 1650165016501650165016);
				assert_eq!(synthetic_balance(), 10891089108910891089106);
				assert_eq!(synthetic_liquidity(), 19108910891089108910894);

				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(2, 1))]));

				// ALICE buy synthetic and BOB sell synthetic
				assert_ok!(synthetic_buy(&ALICE::get(), FEUR, dollar(2000)));
				assert_eq!(collateral_balance(&ALICE::get()), dollar(3000));
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 2640264026402640264025);
				assert_eq!(synthetic_balance(), 13069306930693069306926);
				assert_eq!(synthetic_liquidity(), 18930693069306930693074);
				assert_ok!(synthetic_sell(&BOB::get(), FEUR, dollar(1000)));
				assert_eq!(collateral_balance(&BOB::get()), 6980000000000000000000);
				assert_eq!(multi_currency_balance(&BOB::get(), FEUR), 650165016501650165016);
				assert_eq!(synthetic_balance(), 7238943894389438943890);
				assert_eq!(synthetic_liquidity(), 22781056105610561056110);

				// ALICE sell synthetic and BOB buy synthetic
				assert_ok!(synthetic_sell(&ALICE::get(), FEUR, dollar(1000)));
				assert_eq!(collateral_balance(&ALICE::get()), 4980000000000000000000);
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 1640264026402640264025);
				assert_eq!(synthetic_balance(), 5038943894389438943890);
				assert_eq!(synthetic_liquidity(), 23001056105610561056110);
				assert_ok!(synthetic_buy(&BOB::get(), FEUR, dollar(2000)));
				assert_eq!(collateral_balance(&BOB::get()), 4980000000000000000000);
				assert_eq!(multi_currency_balance(&BOB::get(), FEUR), 1640264026402640264025);
				assert_eq!(synthetic_balance(), 7217161716171617161710);
				assert_eq!(synthetic_liquidity(), 22822838283828382838290);

				assert_ok!(synthetic_sell(
					&ALICE::get(),
					FEUR,
					multi_currency_balance(&ALICE::get(), FEUR)
				));
				assert_ok!(synthetic_sell(
					&BOB::get(),
					FEUR,
					multi_currency_balance(&BOB::get(), FEUR)
				));
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 0);
				assert_eq!(collateral_balance(&ALICE::get()), 8227722772277227722769);
				assert_eq!(multi_currency_balance(&BOB::get(), FEUR), 0);
				assert_eq!(collateral_balance(&BOB::get()), 8227722772277227722769);
				assert_eq!(synthetic_balance(), 0);
				assert_eq!(synthetic_liquidity(), 23544554455445544554462);
			});
	}

	#[test]
	fn test_synthetic_multiple_users_multiple_currencies() {
		ExtBuilder::default()
			.balances(vec![
				(POOL::get(), AUSD, dollar(40_000)),
				(ALICE::get(), AUSD, dollar(10_000)),
				(BOB::get(), AUSD, dollar(10_000)),
			])
			.build()
			.execute_with(|| {
				assert_ok!(synthetic_create_pool());
				assert_ok!(synthetic_set_enabled_trades());
				assert_ok!(synthetic_deposit_liquidity(&POOL::get(), dollar(40_000)));
				assert_ok!(synthetic_set_min_additional_collateral_ratio(Permill::from_percent(10)));
				assert_ok!(synthetic_set_additional_collateral_ratio(
					FEUR,
					Permill::from_percent(10)
				));
				assert_ok!(synthetic_set_additional_collateral_ratio(
					FJPY,
					Permill::from_percent(10)
				));
				assert_ok!(synthetic_set_spread(FEUR, Permill::from_percent(1)));
				assert_ok!(synthetic_set_spread(FJPY, Permill::from_percent(1)));
				assert_ok!(set_oracle_price(vec![
					(FEUR, Price::from_rational(3, 1)),
					(FJPY, Price::from_rational(4, 1))
				]));

				assert_eq!(collateral_balance(&POOL::get()), 0);
				assert_eq!(collateral_balance(&ALICE::get()), dollar(10_000));
				assert_eq!(collateral_balance(&BOB::get()), dollar(10_000));
				assert_eq!(synthetic_liquidity(), dollar(40_000));
				assert_eq!(synthetic_balance(), 0);

				// ALICE buy synthetic FEUR and BOB buy synthetic FJPY
				assert_ok!(synthetic_buy(&ALICE::get(), FEUR, dollar(5000)));
				assert_eq!(collateral_balance(&ALICE::get()), dollar(5000));
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 1650165016501650165016);
				assert_eq!(synthetic_balance(), 5445544554455445544553);
				assert_eq!(synthetic_liquidity(), 39554455445544554455447);

				assert_ok!(synthetic_buy(&BOB::get(), FJPY, dollar(5000)));
				assert_eq!(collateral_balance(&BOB::get()), dollar(5000));
				assert_eq!(multi_currency_balance(&BOB::get(), FJPY), 1237623762376237623762);
				assert_eq!(synthetic_balance(), 10891089108910891089106);
				assert_eq!(synthetic_liquidity(), 39108910891089108910894);

				// change price
				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(2, 1))]));
				assert_ok!(set_oracle_price(vec![(FJPY, Price::from_rational(5, 1))]));

				// ALICE buy synthetic FJPY and BOB sell FEUR
				assert_ok!(synthetic_buy(&ALICE::get(), FJPY, dollar(2000)));
				assert_eq!(collateral_balance(&ALICE::get()), dollar(3000));
				assert_eq!(multi_currency_balance(&ALICE::get(), FJPY), 396039603960396039603);
				assert_eq!(synthetic_balance(), 13069306930693069306922);
				assert_eq!(synthetic_liquidity(), 38930693069306930693078);

				assert_ok!(synthetic_buy(&BOB::get(), FEUR, dollar(2000)));
				assert_eq!(collateral_balance(&BOB::get()), dollar(3000));
				assert_eq!(multi_currency_balance(&BOB::get(), FEUR), 990099009900990099009);
				assert_eq!(synthetic_balance(), 15247524752475247524742);
				assert_eq!(synthetic_liquidity(), 38752475247524752475258);

				// ALICE sell synthetic FEUR and BOB sell synthetic FJPY
				assert_ok!(synthetic_sell(&ALICE::get(), FEUR, dollar(100)));
				assert_eq!(collateral_balance(&ALICE::get()), 3198000000000000000000);
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 1550165016501650165016);
				assert_eq!(synthetic_balance(), 13212343234323432343224);
				assert_eq!(synthetic_liquidity(), 40589656765676567656776);

				assert_ok!(synthetic_sell(&BOB::get(), FJPY, dollar(100)));
				assert_eq!(collateral_balance(&BOB::get()), 3495000000000000000000);
				assert_eq!(multi_currency_balance(&BOB::get(), FJPY), 1137623762376237623762);
				assert_eq!(synthetic_balance(), 12717343234323432343224);
				assert_eq!(synthetic_liquidity(), 40589656765676567656776);

				// ALICE sell synthetic FJPY and BOB sell synthetic FEUR
				assert_ok!(synthetic_sell(&ALICE::get(), FJPY, dollar(100)));
				assert_eq!(collateral_balance(&ALICE::get()), 3693000000000000000000);
				assert_eq!(multi_currency_balance(&ALICE::get(), FJPY), 296039603960396039603);
				assert_eq!(synthetic_balance(), 12222343234323432343224);
				assert_eq!(synthetic_liquidity(), 40589656765676567656776);

				assert_ok!(synthetic_sell(&BOB::get(), FEUR, dollar(100)));
				assert_eq!(collateral_balance(&BOB::get()), 3693000000000000000000);
				assert_eq!(multi_currency_balance(&BOB::get(), FEUR), 890099009900990099009);
				assert_eq!(synthetic_balance(), 12002343234323432343224);
				assert_eq!(synthetic_liquidity(), 40611656765676567656776);
			});
	}

	#[test]
	fn test_synthetic_liquidate_position() {
		ExtBuilder::default()
			.balances(vec![
				(POOL::get(), AUSD, dollar(20_000)),
				(ALICE::get(), AUSD, dollar(10_000)),
			])
			.build()
			.execute_with(|| {
				assert_ok!(synthetic_create_pool());
				assert_ok!(synthetic_set_enabled_trades());
				assert_ok!(synthetic_deposit_liquidity(&POOL::get(), dollar(20_000)));
				assert_ok!(synthetic_set_min_additional_collateral_ratio(Permill::from_percent(10)));
				assert_ok!(synthetic_set_additional_collateral_ratio(
					FEUR,
					Permill::from_percent(10)
				));
				assert_ok!(synthetic_set_spread(FEUR, Permill::from_percent(1)));
				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(3, 1))]));

				assert_ok!(synthetic_buy(&ALICE::get(), FEUR, dollar(5000)));

				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(300, 90))]));

				assert_ok!(synthetic_liquidate(
					&ALICE::get(),
					FEUR,
					multi_currency_balance(&ALICE::get(), FEUR)
				));
				assert_eq!(synthetic_liquidity(), 19554455445544554455447);
				assert_eq!(collateral_balance(&ALICE::get()), 10445544554455445544552);
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 0);
				assert_eq!(synthetic_balance(), 0);
				assert_eq!(synthetic_liquidity(), 19554455445544554455447);
			});
	}

	#[test]
	fn test_synthetic_add_collateral() {
		ExtBuilder::default()
			.balances(vec![
				(POOL::get(), AUSD, dollar(40_000)),
				(ALICE::get(), AUSD, dollar(10_000)),
			])
			.build()
			.execute_with(|| {
				assert_ok!(synthetic_create_pool());
				assert_ok!(synthetic_set_enabled_trades());
				assert_ok!(synthetic_deposit_liquidity(&POOL::get(), dollar(20_000)));
				assert_ok!(synthetic_set_min_additional_collateral_ratio(Permill::from_percent(10)));
				assert_ok!(synthetic_set_additional_collateral_ratio(
					FEUR,
					Permill::from_percent(1)
				));
				assert_ok!(synthetic_set_spread(FEUR, Permill::from_percent(1)));
				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(3, 1))]));

				assert_ok!(synthetic_buy(&ALICE::get(), FEUR, dollar(5000)));

				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(300, 90))]));

				assert_ok!(synthetic_liquidate(&ALICE::get(), FEUR, 1));
				assert_ok!(synthetic_add_collateral(&POOL::get(), FEUR, dollar(20_000)));
				assert_noop!(
					synthetic_liquidate(&ALICE::get(), FEUR, 1),
					synthetic_protocol::Error::<Runtime>::StillInSafePosition
				);
			});
	}

	#[test]
	fn test_synthetic_liquidate_partially() {
		ExtBuilder::default()
			.balances(vec![
				(POOL::get(), AUSD, dollar(20_000)),
				(ALICE::get(), AUSD, dollar(10_000)),
			])
			.build()
			.execute_with(|| {
				assert_ok!(synthetic_create_pool());
				assert_ok!(synthetic_set_enabled_trades());
				assert_ok!(synthetic_deposit_liquidity(&POOL::get(), dollar(20_000)));
				assert_ok!(synthetic_set_min_additional_collateral_ratio(Permill::from_percent(10)));
				assert_ok!(synthetic_set_additional_collateral_ratio(
					FEUR,
					Permill::from_percent(10)
				));
				assert_ok!(synthetic_set_spread(FEUR, Permill::from_percent(1)));
				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(3, 1))]));

				assert_ok!(synthetic_buy(&ALICE::get(), FEUR, dollar(5000)));

				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(300, 90))]));

				assert_ok!(synthetic_liquidate(&ALICE::get(), FEUR, dollar(800)));
				assert_eq!(collateral_balance(&ALICE::get()), 7640000000000000000000);
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 850165016501650165016);
				assert_eq!(synthetic_balance(), 2805544554455445544553);
				assert_eq!(synthetic_liquidity(), 19554455445544554455447);

				assert_ok!(synthetic_liquidate(
					&ALICE::get(),
					FEUR,
					multi_currency_balance(&ALICE::get(), FEUR)
				));
				assert_eq!(collateral_balance(&ALICE::get()), 10445544554455445544552);
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 0);
				assert_eq!(synthetic_balance(), 0);
				assert_eq!(synthetic_liquidity(), 19554455445544554455447);
				assert_ok!(synthetic_withdraw_liquidity(&POOL::get(), dollar(1000)));
				assert_eq!(synthetic_liquidity(), 18554455445544554455447);
			});
	}

	#[test]
	fn test_synthetic_liquidate_remove() {
		ExtBuilder::default()
			.balances(vec![
				(POOL::get(), AUSD, dollar(20_000)),
				(ALICE::get(), AUSD, dollar(10_000)),
			])
			.build()
			.execute_with(|| {
				assert_ok!(synthetic_create_pool());
				assert_ok!(synthetic_set_enabled_trades());
				assert_ok!(synthetic_deposit_liquidity(&POOL::get(), dollar(20_000)));
				assert_ok!(synthetic_set_min_additional_collateral_ratio(Permill::from_percent(10)));
				assert_ok!(synthetic_set_additional_collateral_ratio(
					FEUR,
					Permill::from_percent(10)
				));
				assert_ok!(synthetic_set_spread(FEUR, Permill::from_percent(1)));
				assert_ok!(set_oracle_price(vec![(FEUR, Price::from_rational(3, 1))]));

				assert_ok!(synthetic_buy(&ALICE::get(), FEUR, dollar(5000)));
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 1650165016501650165016);
				assert_noop!(
					synthetic_remove_pool(&POOL::get()),
					synthetic_liquidity_pools::Error::<Runtime>::CannotRemovePool
				);

				assert_ok!(synthetic_sell(
					&ALICE::get(),
					FEUR,
					multi_currency_balance(&ALICE::get(), FEUR)
				));
				assert_eq!(multi_currency_balance(&ALICE::get(), FEUR), 0);
				assert_ok!(synthetic_disable_pool(&POOL::get()));
				assert_ok!(synthetic_remove_pool(&POOL::get()));
			});
	}
}
