#![cfg(test)]

use super::*;
use mock::*;

use frame_support::{assert_noop, assert_ok};
use traits::LiquidityPools;

#[test]
fn is_owner_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Instance1Module::create_pool(Origin::signed(ALICE)));
		assert!(Instance1Module::is_owner(0, &ALICE));
		assert!(!Instance1Module::is_owner(1, &ALICE));
		assert!(!<Instance1Module as LiquidityPools<AccountId>>::is_owner(1, &ALICE));
	});
}

#[test]
fn should_create_pool() {
	new_test_ext().execute_with(|| {
		assert_ok!(Instance1Module::create_pool(Origin::signed(ALICE)));
		assert_eq!(Instance1Module::owners(0), Some((ALICE, 0)));
		assert_eq!(Instance1Module::next_pool_id(), 1);
	});
}

#[test]
fn should_disable_pool() {
	new_test_ext().execute_with(|| {
		assert_ok!(Instance1Module::create_pool(Origin::signed(ALICE)));
		assert_ok!(Instance1Module::disable_pool(Origin::signed(ALICE), 0));
	})
}

#[test]
fn should_remove_pool() {
	new_test_ext().execute_with(|| {
		assert_ok!(Instance1Module::create_pool(Origin::signed(ALICE)));
		assert_ok!(Instance1Module::deposit_liquidity(Origin::signed(ALICE), 0, 1000));
		assert_eq!(Instance1Module::balances(&0), 1000);
		assert_ok!(Instance1Module::remove_pool(Origin::signed(ALICE), 0));
		assert_eq!(Instance1Module::owners(0), None);
		assert_eq!(Instance1Module::balances(&0), 0);
		assert_eq!(<Instance1Module as LiquidityPools<AccountId>>::liquidity(0), 0);
	})
}

#[test]
fn should_deposit_liquidity() {
	new_test_ext().execute_with(|| {
		assert_ok!(Instance1Module::create_pool(Origin::signed(ALICE)));
		assert_eq!(Instance1Module::balances(&0), 0);
		assert_ok!(Instance1Module::deposit_liquidity(Origin::signed(ALICE), 0, 1000));
		assert_eq!(Instance1Module::balances(&0), 1000);
		assert_eq!(<Instance1Module as LiquidityPools<AccountId>>::liquidity(0), 1000);
		assert_noop!(
			Instance1Module::deposit_liquidity(Origin::signed(ALICE), 1, 1000),
			Error::<Runtime, Instance1>::PoolNotFound
		);
	})
}

#[test]
fn should_withdraw_liquidity() {
	new_test_ext().execute_with(|| {
		assert_ok!(Instance1Module::create_pool(Origin::signed(ALICE)));
		assert_eq!(Instance1Module::owners(0), Some((ALICE, 0)));
		assert_eq!(Instance1Module::balances(&0), 0);
		assert_ok!(Instance1Module::deposit_liquidity(Origin::signed(ALICE), 0, 1000));
		assert_eq!(Instance1Module::balances(&0), 1000);
		assert_ok!(Instance1Module::withdraw_liquidity(Origin::signed(ALICE), 0, 500));
		assert_eq!(Instance1Module::balances(&0), 500);
		assert_ok!(<Instance1Module as LiquidityPools<AccountId>>::withdraw_liquidity(
			&BOB, 0, 100
		));
		assert_eq!(Instance1Module::balances(&0), 400);
	})
}

#[test]
fn should_fail_withdraw_liquidity() {
	new_test_ext().execute_with(|| {
		assert_ok!(Instance1Module::create_pool(Origin::signed(ALICE)));
		assert_ok!(Instance1Module::deposit_liquidity(Origin::signed(ALICE), 0, 1000));
		assert_eq!(Instance1Module::balances(&0), 1000);
		assert_eq!(
			Instance1Module::withdraw_liquidity(Origin::signed(ALICE), 0, 5000),
			Err(Error::<Runtime, Instance1>::CannotWithdrawAmount.into()),
		);

		assert_eq!(
			Instance1Module::withdraw_liquidity(Origin::signed(ALICE), 0, 1000),
			Err(Error::<Runtime, Instance1>::CannotWithdrawExistentialDeposit.into()),
		);

		assert_eq!(Instance1Module::balances(&0), 1000);
	})
}

#[test]
fn multi_instances_have_independent_storage() {
	new_test_ext().execute_with(|| {
		// owners storage
		assert_ok!(Instance1Module::create_pool(Origin::signed(ALICE)));
		assert_eq!(Instance1Module::all(), vec![0]);
		assert_eq!(Instance2Module::all(), vec![]);
		// pool id storage
		assert_eq!(Instance1Module::next_pool_id(), 1);
		assert_eq!(Instance2Module::next_pool_id(), 0);

		assert_ok!(Instance2Module::create_pool(Origin::signed(ALICE)));

		// balances storage
		assert_ok!(Instance1Module::deposit_liquidity(Origin::signed(ALICE), 0, 1000));
		assert_eq!(Instance1Module::balances(&0), 1000);
		assert_eq!(LiquidityCurrency::free_balance(&Instance1Module::account_id()), 1000);
		assert_eq!(Instance2Module::balances(&0), 0);
		assert_eq!(LiquidityCurrency::free_balance(&Instance2Module::account_id()), 0);
	})
}