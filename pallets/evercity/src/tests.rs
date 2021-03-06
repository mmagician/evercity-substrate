#![allow(unused_imports)]
use crate::mock::*;
use crate::{
    AccountRegistry, BondId, BondImpactReportStruct, BondImpactReportStructOf, BondInnerStructOf,
    BondState, BondStructOf, Error, Event, EverUSDBalance, Module, AUDITOR_ROLE_MASK,
    CUSTODIAN_ROLE_MASK, DAY_DURATION, EMITENT_ROLE_MASK, INVESTOR_ROLE_MASK, MASTER_ROLE_MASK,
};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchResult, Blake2_256, StorageHasher};
use frame_system::Trait;

type Evercity = Module<TestRuntime>;
type Timestamp = pallet_timestamp::Module<TestRuntime>;
type Moment = <TestRuntime as pallet_timestamp::Trait>::Moment;
type BondInnerStruct = BondInnerStructOf<TestRuntime>;
type BondStruct = BondStructOf<TestRuntime>;
type RuntimeError = Error<TestRuntime>;
type AccountId = <TestRuntime as frame_system::Trait>::AccountId;

//////////////////////////////////////////////////////////////////////////////////////////////////////////
// Test uses pack of accounts, pre-set in new_test_ext in mock.rs:
// (1, EvercityAccountStruct { roles: MASTER_ROLE_MASK,     identity: 10u64}), // MASTER    (accountId: 1)
// (2, EvercityAccountStruct { roles: CUSTODIAN_ROLE_MASK,  identity: 20u64}), // CUSTODIAN (accountID: 2)
// (3, EvercityAccountStruct { roles: EMITENT_ROLE_MASK,    identity: 30u64}), // EMITENT   (accountID: 3)
// (4, EvercityAccountStruct { roles: INVESTOR_ROLE_MASK,   identity: 40u64}), // INVESTOR  (accountId: 4)
// (5, EvercityAccountStruct { roles: AUDITOR_ROLE_MASK,    identity: 50u64}), // AUDITOR   (accountId: 5)
// (7, EvercityAccountStruct { roles: EMITENT_ROLE_MASK,    identity: 70u64}), // EMITENT   (accountId: 5)
// (8, EvercityAccountStruct { roles: MANAGER_ROLE_MASK,    identity: 80u64}), // MANAGER   (accountId: 8)
// (101+ : some external accounts
//////////////////////////////////////////////////////////////////////////////////////////////////////////

fn bond_current_period(bond: &BondStruct, now: Moment) -> u32 {
    bond.time_passed_after_activation(now).unwrap().1
}

const CUSTODIAN_ID: u64 = 2;

#[test]
fn it_returns_true_for_correct_role_checks() {
    new_test_ext().execute_with(|| {
        assert_eq!(Evercity::account_is_master(&1), true);
        assert_eq!(Evercity::account_is_custodian(&2), true);
        assert_eq!(Evercity::account_is_emitent(&3), true);
        assert_eq!(Evercity::account_is_investor(&4), true);
        assert_eq!(Evercity::account_is_auditor(&5), true);
    });
}

#[test]
fn it_returns_false_for_incorrect_role_checks() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        //assert_ok!(AccountRegistry::insert(Origin::signed(1), EvercityAccountStruct {roles: 1u8, identity: 67u64}));
        // Read pallet storage and assert an expected result.
        assert_eq!(Evercity::account_is_auditor(&1), false);
        assert_eq!(Evercity::account_is_emitent(&2), false);
        assert_eq!(Evercity::account_is_investor(&3), false);
        assert_eq!(Evercity::account_is_custodian(&4), false);
        assert_eq!(Evercity::account_is_master(&5), false);
    });
}

#[test]
fn it_adds_new_account_with_correct_roles() {
    new_test_ext().execute_with(|| {
        Timestamp::set_timestamp(12345);

        assert_ok!(Evercity::account_add_with_role_and_data(
            Origin::signed(1),
            101,
            MASTER_ROLE_MASK,
            88u64
        ));
        assert_eq!(Evercity::account_is_master(&101), true);
        assert_eq!(Evercity::account_is_investor(&101), false);

        assert_ok!(Evercity::account_add_with_role_and_data(
            Origin::signed(1),
            102,
            AUDITOR_ROLE_MASK,
            89u64
        ));
        assert_eq!(Evercity::account_is_master(&102), false);
        assert_eq!(Evercity::account_is_auditor(&102), true);
    });
}
#[test]
fn it_correctly_sets_new_role_to_existing_account() {
    new_test_ext().execute_with(|| {
        // add new role to existing account (alowed only for master)
        assert_eq!(Evercity::account_is_emitent(&3), true);
        assert_ok!(Evercity::account_set_with_role_and_data(
            Origin::signed(1),
            3,
            AUDITOR_ROLE_MASK,
            88u64
        ));
        assert_eq!(Evercity::account_is_emitent(&3), true);
        assert_eq!(Evercity::account_is_auditor(&3), true);
        assert_eq!(Evercity::account_is_investor(&3), false);

        assert_eq!(Evercity::account_is_custodian(&2), true);
        assert_eq!(Evercity::account_is_emitent(&2), false);
        assert_ok!(Evercity::account_set_with_role_and_data(
            Origin::signed(1),
            2,
            EMITENT_ROLE_MASK,
            89u64
        ));
        assert_eq!(Evercity::account_is_custodian(&2), true);
        assert_eq!(Evercity::account_is_emitent(&2), true);
    });
}

#[test]
fn it_disable_account() {
    new_test_ext().execute_with(|| {
        assert_ok!(Evercity::account_add_with_role_and_data(
            Origin::signed(1),
            101,
            MASTER_ROLE_MASK,
            88u64
        ));
        assert_eq!(Evercity::account_is_master(&101), true);
        assert_ok!(Evercity::account_disable(Origin::signed(1), 101));

        assert_eq!(Evercity::account_is_master(&101), false);
    });
}

#[test]
fn it_denies_add_and_set_roles_for_non_master() {
    new_test_ext().execute_with(|| {
        // trying to add account form non-master account
        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(12345);
        assert_noop!(
            Evercity::account_add_with_role_and_data(
                Origin::signed(2),
                101,
                MASTER_ROLE_MASK,
                88u64
            ),
            RuntimeError::AccountNotAuthorized
        );

        assert_noop!(
            Evercity::account_set_with_role_and_data(
                Origin::signed(3),
                3,
                EMITENT_ROLE_MASK,
                88u64
            ),
            RuntimeError::AccountNotAuthorized
        );
    });
}

// mint tokens

#[test]
fn it_token_mint_create_with_confirm() {
    const ACCOUNT: u64 = 4; // INVESTOR
    new_test_ext().execute_with(|| {
        assert_ok!(Evercity::token_mint_request_create_everusd(
            Origin::signed(ACCOUNT),
            100000
        ));

        assert_eq!(Evercity::total_supply(), 0);

        assert_ok!(Evercity::token_mint_request_confirm_everusd(
            Origin::signed(CUSTODIAN_ID),
            ACCOUNT,
            100000
        ));

        assert_eq!(Evercity::total_supply(), 100000);
    });
}

#[test]
fn it_token_mint_create_with_revoke() {
    const ACCOUNT: u64 = 4; // INVESTOR
    new_test_ext().execute_with(|| {
        assert_ok!(Evercity::token_mint_request_create_everusd(
            Origin::signed(ACCOUNT), // INVESTOR
            100000
        ));

        assert_ok!(Evercity::token_mint_request_revoke_everusd(Origin::signed(
            ACCOUNT
        ),));

        assert_noop!(
            Evercity::token_mint_request_confirm_everusd(
                Origin::signed(CUSTODIAN_ID),
                ACCOUNT,
                100000
            ),
            RuntimeError::MintRequestDoesntExist
        );
    });
}

#[test]
fn it_token_mint_create_with_decline() {
    const ACCOUNT: u64 = 4; // INVESTOR
    new_test_ext().execute_with(|| {
        assert_ok!(Evercity::token_mint_request_create_everusd(
            Origin::signed(ACCOUNT),
            100000
        ));

        assert_ok!(Evercity::token_mint_request_decline_everusd(
            Origin::signed(CUSTODIAN_ID),
            ACCOUNT
        ));

        assert_noop!(
            Evercity::token_mint_request_revoke_everusd(Origin::signed(ACCOUNT)),
            RuntimeError::MintRequestDoesntExist
        );
    });
}

#[test]
fn it_token_mint_create_denied() {
    const ACCOUNT: u64 = 5; // AUDITOR
    new_test_ext().execute_with(|| {
        assert_noop!(
            Evercity::token_mint_request_create_everusd(Origin::signed(ACCOUNT), 100000),
            RuntimeError::AccountNotAuthorized
        );
    });
}

#[test]
fn it_token_mint_create_hasty() {
    const ACCOUNT: u64 = 4; // INVESTOR
    new_test_ext().execute_with(|| {
        assert_ok!(Evercity::token_mint_request_create_everusd(
            Origin::signed(ACCOUNT),
            100000
        ));

        assert_noop!(
            Evercity::token_mint_request_create_everusd(Origin::signed(ACCOUNT), 10),
            RuntimeError::MintRequestAlreadyExist
        );
    });
}

#[test]
fn it_token_mint_create_toolarge() {
    const ACCOUNT: u64 = 4;
    new_test_ext().execute_with(|| {
        assert_noop!(
            Evercity::token_mint_request_create_everusd(
                Origin::signed(ACCOUNT), // INVESTOR
                crate::EVERUSD_MAX_MINT_AMOUNT + 1
            ),
            RuntimeError::MintRequestParamIncorrect
        );
    });
}

// burn tokens

fn add_token(id: AccountId, amount: EverUSDBalance) -> DispatchResult {
    Evercity::token_mint_request_create_everusd(Origin::signed(id), amount)?;

    Evercity::token_mint_request_confirm_everusd(Origin::signed(CUSTODIAN_ID), id, amount)
}

#[test]
fn it_token_burn_create_with_confirm() {
    const ACCOUNT: u64 = 4; // INVESTOR

    new_test_ext().execute_with(|| {
        assert_ok!(add_token(ACCOUNT, 10000));

        assert_ok!(Evercity::token_burn_request_create_everusd(
            Origin::signed(ACCOUNT),
            10000
        ));

        assert_eq!(Evercity::total_supply(), 10000);

        assert_ok!(Evercity::token_burn_request_confirm_everusd(
            Origin::signed(CUSTODIAN_ID),
            ACCOUNT,
            10000
        ));

        assert_eq!(Evercity::total_supply(), 0);
        // duplicate confirmations is not allowed
        assert_noop!(
            Evercity::token_burn_request_confirm_everusd(
                Origin::signed(CUSTODIAN_ID),
                ACCOUNT,
                10000
            ),
            RuntimeError::BurnRequestDoesntExist
        );
    });
}

#[test]
fn it_token_burn_create_overrun() {
    const ACCOUNT: u64 = 3; // EMITENT
    const BALANCE: EverUSDBalance = 10000;

    new_test_ext().execute_with(|| {
        assert_ok!(add_token(ACCOUNT, BALANCE));

        assert_noop!(
            Evercity::token_burn_request_create_everusd(Origin::signed(ACCOUNT), BALANCE + 1),
            RuntimeError::MintRequestParamIncorrect
        );
    });
}

#[test]
fn it_token_burn_create_with_revoke() {
    const ACCOUNT: u64 = 3; // EMITENT

    new_test_ext().execute_with(|| {
        assert_ok!(add_token(ACCOUNT, 10000));

        assert_ok!(Evercity::token_burn_request_create_everusd(
            Origin::signed(ACCOUNT),
            10000
        ));

        assert_ok!(Evercity::token_burn_request_revoke_everusd(Origin::signed(
            ACCOUNT
        ),));

        assert_noop!(
            Evercity::token_burn_request_confirm_everusd(
                Origin::signed(CUSTODIAN_ID),
                ACCOUNT,
                10000
            ),
            RuntimeError::BurnRequestDoesntExist
        );
    });
}

fn get_test_bond() -> BondStruct {
    BondStruct {
        inner: BondInnerStruct {
            docs_pack_root_hash_main: Default::default(),
            docs_pack_root_hash_legal: Default::default(),
            docs_pack_root_hash_finance: Default::default(),
            docs_pack_root_hash_tech: Default::default(),

            impact_data_type: Default::default(),
            impact_data_baseline: 20000_u64,
            impact_data_max_deviation_cap: 30000_u64,
            impact_data_max_deviation_floor: 14000_u64,
            interest_rate_penalty_for_missed_report: 400, //0.4%

            interest_rate_base_value: 2000,   // 2.0%
            interest_rate_margin_cap: 4000,   // 4.0%
            interest_rate_margin_floor: 1000, //1%
            interest_rate_start_period_value: 1900,
            start_period: 120 * DAY_DURATION,
            payment_period: 30 * DAY_DURATION,     // every month
            interest_pay_period: 7 * DAY_DURATION, // up to 7 days after the new period started
            mincap_deadline: (20 * DAY_DURATION * 1000) as u64,
            impact_data_send_period: 10 * DAY_DURATION, // 10 days before next period
            bond_duration: 12,                          //
            bond_finishing_period: 14 * DAY_DURATION,

            bond_units_mincap_amount: 1000,
            bond_units_maxcap_amount: 1800,
            bond_units_base_price: 4_000_000_000_000,
        },

        emitent: 0,
        manager: 0,
        auditor: 0,
        impact_reporter: 0,

        issued_amount: 0,
        booking_start_date: Default::default(),
        active_start_date: Default::default(),
        creation_date: Default::default(),
        state: Default::default(),

        bond_debit: 0,
        bond_credit: 0,
        coupon_yield: 0,
    }
}

#[test]
fn bond_validation() {
    new_test_ext().execute_with(|| {
        let bond = get_test_bond();
        assert_eq!(bond.inner.is_valid(), true);
    });
}

#[test]
fn bond_equation() {
    new_test_ext().execute_with(|| {
        let bond1 = get_test_bond();

        let mut bond2 = bond1.clone();
        assert_eq!(bond1.inner, bond2.inner);
        bond2.inner.docs_pack_root_hash_legal = Blake2_256::hash(b"").into();

        assert!(bond1.inner.is_financial_options_eq(&bond2.inner));
        assert_ne!(bond1.inner, bond2.inner);

        bond2.inner.docs_pack_root_hash_legal = bond1.inner.docs_pack_root_hash_legal;
        bond2.inner.payment_period += 1;

        assert!(!bond1.inner.is_financial_options_eq(&bond2.inner));
        assert_ne!(bond1.inner, bond2.inner);
    });
}

#[test]
fn bond_interest_min_max() {
    new_test_ext().execute_with(|| {
        let bond = get_test_bond();
        // full amplitude
        assert_eq!(
            bond.interest_rate(bond.inner.impact_data_baseline),
            bond.inner.interest_rate_base_value
        );
        assert_eq!(
            bond.interest_rate(bond.inner.impact_data_max_deviation_cap),
            bond.inner.interest_rate_margin_floor
        );
        assert_eq!(
            bond.interest_rate(bond.inner.impact_data_max_deviation_cap + 1),
            bond.inner.interest_rate_margin_floor
        );
        assert_eq!(
            bond.interest_rate(bond.inner.impact_data_max_deviation_floor),
            bond.inner.interest_rate_margin_cap
        );
        assert_eq!(
            bond.interest_rate(bond.inner.impact_data_max_deviation_floor - 1),
            bond.inner.interest_rate_margin_cap
        );

        // partial amplitude
        assert_eq!(bond.interest_rate(25000_u64), 1500);
        assert_eq!(bond.interest_rate(29000_u64), 1100);

        assert_eq!(bond.interest_rate(17000_u64), 3000);
        assert_eq!(bond.interest_rate(15000_u64), 3666);
    });
}

#[test]
fn bond_period_interest_rate() {
    new_test_ext().execute_with(|| {
        let bond = get_test_bond();

        assert_eq!(bond.inner.impact_data_baseline, 20000_u64);

        let mut reports = Vec::<BondImpactReportStructOf<TestRuntime>>::new();
        //missing report
        reports.push(BondImpactReportStructOf::<TestRuntime> {
            create_date: 0,
            impact_data: 0,
            signed: false,
        });
        reports.push(BondImpactReportStructOf::<TestRuntime> {
            create_date: 0,
            impact_data: 20000_u64,
            signed: true,
        });
        //missing report
        reports.push(BondImpactReportStructOf::<TestRuntime> {
            create_date: 0,
            impact_data: 0,
            signed: false,
        });
        // worst result and maximal interest rate value
        reports.push(BondImpactReportStructOf::<TestRuntime> {
            create_date: 0,
            impact_data: 14000_u64,
            signed: true,
        });
        //missing report. it cannot make interest rate worse
        reports.push(BondImpactReportStructOf::<TestRuntime> {
            create_date: 0,
            impact_data: 0,
            signed: false,
        });
        // very good result lead to mininal interest rate
        reports.push(BondImpactReportStructOf::<TestRuntime> {
            create_date: 0,
            impact_data: 100000_u64,
            signed: true,
        });
        //first missing report.
        reports.push(BondImpactReportStructOf::<TestRuntime> {
            create_date: 0,
            impact_data: 0,
            signed: false,
        });
        //second missing report.
        reports.push(BondImpactReportStructOf::<TestRuntime> {
            create_date: 0,
            impact_data: 0,
            signed: false,
        });

        assert_eq!(
            bond.inner.interest_rate_start_period_value,
            Evercity::calc_bond_interest_rate(&bond, reports.as_ref(), 0)
        );

        assert_eq!(
            bond.inner.interest_rate_start_period_value
                + bond.inner.interest_rate_penalty_for_missed_report,
            Evercity::calc_bond_interest_rate(&bond, reports.as_ref(), 1)
        );

        assert_eq!(
            bond.inner.interest_rate_base_value,
            Evercity::calc_bond_interest_rate(&bond, reports.as_ref(), 2)
        );

        assert_eq!(
            bond.inner.interest_rate_base_value
                + bond.inner.interest_rate_penalty_for_missed_report,
            Evercity::calc_bond_interest_rate(&bond, reports.as_ref(), 3)
        );

        assert_eq!(
            bond.inner.interest_rate_margin_cap,
            Evercity::calc_bond_interest_rate(&bond, reports.as_ref(), 4)
        );
        // missing report cannot increase insterested rate above maximal value
        assert_eq!(
            bond.inner.interest_rate_margin_cap,
            Evercity::calc_bond_interest_rate(&bond, reports.as_ref(), 5)
        );

        assert_eq!(
            bond.inner.interest_rate_margin_floor,
            Evercity::calc_bond_interest_rate(&bond, reports.as_ref(), 6)
        );

        assert_eq!(
            bond.inner.interest_rate_margin_floor
                + bond.inner.interest_rate_penalty_for_missed_report,
            Evercity::calc_bond_interest_rate(&bond, reports.as_ref(), 7)
        );

        assert_eq!(
            bond.inner.interest_rate_margin_floor
                + 2 * bond.inner.interest_rate_penalty_for_missed_report,
            Evercity::calc_bond_interest_rate(&bond, reports.as_ref(), 8)
        );
    });
}

#[test]
fn bond_basic_calc_coupon_yield() {
    const ACCOUNT: u64 = 3;
    let bondid: BondId = "BOND2".into();

    new_test_ext().execute_with(|| {
        bond_grand_everusd();
        bond_activate(bondid, ACCOUNT, get_test_bond().inner);

        let mut chain_bond_item = Evercity::get_bond(&bondid);

        assert_eq!(chain_bond_item.active_start_date, 30000);
        // pass first (index=0) period
        let mut moment: Moment =
            30000_u64 + (chain_bond_item.inner.start_period) as u64 * 1000_u64 + 1_u64;
        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(moment);

        assert_eq!(bond_current_period(&chain_bond_item, moment), 1);
        assert!(Evercity::calc_and_store_bond_coupon_yield(
            &bondid,
            &mut chain_bond_item,
            moment
        ));
        // second call should return false
        assert!(!Evercity::calc_and_store_bond_coupon_yield(
            &bondid,
            &mut chain_bond_item,
            moment
        ));

        // pass second (index=1) period
        moment += chain_bond_item.inner.payment_period as u64 * 1000_u64;
        assert_eq!(bond_current_period(&chain_bond_item, moment), 2);
        chain_bond_item.bond_debit = 2000;

        assert!(Evercity::calc_and_store_bond_coupon_yield(
            &bondid,
            &mut chain_bond_item,
            moment
        ));

        let bond_yields = Evercity::get_coupon_yields(&bondid);

        assert_eq!(bond_yields.len(), 2);
        assert_eq!(
            bond_yields[0].interest_rate,
            chain_bond_item.inner.interest_rate_start_period_value
        );
        assert_eq!(bond_yields[0].total_yield, 29983561640400);
        // 29982 = 4000(price) * (600+600)(count) * 120(days) / 365 * 1900(interest)/100000

        assert_eq!(
            bond_yields[1].interest_rate,
            chain_bond_item.inner.interest_rate_start_period_value
                + chain_bond_item
                    .inner
                    .interest_rate_penalty_for_missed_report
        );
        assert_eq!(bond_yields[1].total_yield, 39057534239800);
        // 9072 = 4000 * (600) * 30 / 365 * (1900+400)/100000  x 2
    });
}

#[test]
fn bond_advanced_calc_coupon_yield() {
    const ACCOUNT1: u64 = 3;
    const ACCOUNT2: u64 = 7;
    const INVESTOR1: u64 = 4;
    const INVESTOR2: u64 = 6;
    let bondid1: BondId = "BOND1".into();
    let bondid2: BondId = "BOND2".into();

    //         |-  investor1 600 presale + 400 after 100 days ( during start period ),
    // bond1---|
    //         |-  investor2 600 presale

    //         |-  investor1 600 presale
    // bond2---|
    //         |-  investor2 600 presale + 400 after 140 days (second period )

    new_test_ext().execute_with(|| {
        bond_grand_everusd();
        bond_activate(bondid1, ACCOUNT1, get_test_bond().inner);
        bond_activate(bondid2, ACCOUNT2, get_test_bond().inner);

        let chain_bond_item1 = Evercity::get_bond(&bondid1);
        let chain_bond_item2 = Evercity::get_bond(&bondid2);

        let start_moment = chain_bond_item1.active_start_date;
        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(
            start_moment + (100 * DAY_DURATION) as u64 * 1000,
        );

        assert_ok!(Evercity::bond_unit_package_buy(
            Origin::signed(INVESTOR1),
            bondid1,
            400
        ));

        assert_eq!(
            bond_current_period(
                &chain_bond_item1,
                start_moment + (100 * DAY_DURATION) as u64 * 1000
            ),
            0
        );

        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(
            start_moment + (140 * DAY_DURATION) as u64 * 1000,
        );
        assert_ok!(Evercity::bond_unit_package_buy(
            Origin::signed(INVESTOR2),
            bondid2,
            400
        ));

        assert_eq!(
            bond_current_period(
                &chain_bond_item2,
                start_moment + (140 * DAY_DURATION) as u64 * 1000
            ),
            1
        );

        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(
            start_moment + (160 * DAY_DURATION) as u64 * 1000,
        );

        assert_eq!(
            bond_current_period(
                &chain_bond_item2,
                start_moment + (160 * DAY_DURATION) as u64 * 1000
            ),
            2
        );

        let _investor1_balance = Evercity::balance_everusd(&INVESTOR1);
        let _investor2_balance = Evercity::balance_everusd(&INVESTOR2);
        // set impact data
        assert_ok!(Evercity::set_impact_data(
            &bondid1,
            0,
            chain_bond_item1.inner.impact_data_baseline
        ));
        //Evercity::set_impact_data(&bondid1, 1, chain_bond_item1.inner.impact_data_baseline );
        assert_ok!(Evercity::set_impact_data(
            &bondid2,
            0,
            chain_bond_item2.inner.impact_data_baseline
        ));

        // request coupon yield
    });
}

#[test]
fn bond_periods() {
    let mut bond = get_test_bond();
    bond.state = BondState::ACTIVE;
    bond.active_start_date += 10;

    assert_eq!(bond.time_passed_after_activation(0), None);
    assert_eq!(
        bond.time_passed_after_activation(bond.active_start_date),
        Some((0, 0))
    );
    let start_period = bond.active_start_date + 120 * 1000 * DAY_DURATION as u64;
    assert_eq!(bond.inner.start_period, 120 * DAY_DURATION);

    assert_eq!(
        bond.time_passed_after_activation(start_period),
        Some((120 * DAY_DURATION, 1))
    );
    assert_eq!(
        bond.time_passed_after_activation(start_period - 1),
        Some((120 * DAY_DURATION - 1, 0))
    );

    assert_eq!(bond.inner.payment_period, 30 * DAY_DURATION);
    assert_eq!(
        bond.time_passed_after_activation(start_period + 30 * 1000 * DAY_DURATION as u64),
        Some(((120 + 30) * DAY_DURATION, 2))
    );
    assert_eq!(
        bond.time_passed_after_activation(start_period + 29 * 1000 * DAY_DURATION as u64),
        Some(((120 + 29) * DAY_DURATION, 1))
    );
    assert_eq!(
        bond.time_passed_after_activation(start_period + 1000 * DAY_DURATION as u64),
        Some(((120 + 1) * DAY_DURATION, 1))
    );
    assert_eq!(
        bond.time_passed_after_activation(start_period + 31 * 1000 * DAY_DURATION as u64),
        Some(((31 + 120) * DAY_DURATION, 2))
    );
    assert_eq!(
        bond.time_passed_after_activation(start_period + 310 * 1000 * DAY_DURATION as u64),
        Some(((120 + 310) * DAY_DURATION, 11))
    );

    assert_eq!(
        bond.time_passed_after_activation(4294967295000),
        Some((4294967294, 13))
    );

    assert_eq!(bond.time_passed_after_activation(6300000000000), None);
}

#[test]
fn bond_try_create_with_same_id() {
    let bond = get_test_bond();
    let bondid: BondId = "TEST".into();
    const ACCOUNT: u64 = 3;

    new_test_ext().execute_with(|| {
        assert_ok!(Evercity::bond_add_new(
            Origin::signed(ACCOUNT),
            bondid,
            bond.inner.clone()
        ));
        assert_noop!(
            Evercity::bond_add_new(Origin::signed(ACCOUNT), bondid, bond.inner.clone()),
            RuntimeError::BondAlreadyExists
        );
        assert_ok!(Evercity::bond_revoke(Origin::signed(ACCOUNT), bondid));
        assert_ok!(Evercity::bond_add_new(
            Origin::signed(ACCOUNT),
            bondid,
            bond.inner.clone()
        ));
    });
}

#[test]
fn bond_create_delete() {
    let bond = get_test_bond();
    let bondid: BondId = "TEST".into();

    const ACCOUNT: u64 = 3;
    new_test_ext().execute_with(|| {
        assert_ok!(Evercity::bond_add_new(
            Origin::signed(ACCOUNT),
            bondid,
            bond.inner.clone()
        ));
        let chain_bond_item = Evercity::get_bond(&bondid);
        assert_eq!(bond.inner, chain_bond_item.inner);

        assert_ok!(Evercity::bond_revoke(Origin::signed(ACCOUNT), bondid));
        let chain_bond_item = Evercity::get_bond(&bondid);
        assert_ne!(bond.inner, chain_bond_item.inner);
        assert_eq!(chain_bond_item.inner, Default::default());
    });
}

fn bond_grand_everusd() {
    const INVESTOR1: u64 = 4;
    const INVESTOR2: u64 = 6;

    assert_ok!(add_token(INVESTOR1, 50_000_000_000_000_000));
    assert_ok!(add_token(INVESTOR2, 50_000_000_000_000_000));
}

fn bond_activate(bondid: BondId, acc: u64, mut bond: BondInnerStruct) {
    const MASTER: u64 = 1;
    const AUDITOR: u64 = 5;
    const INVESTOR1: u64 = 4;
    const INVESTOR2: u64 = 6;

    let investor1_balance = Evercity::balance_everusd(&INVESTOR1);
    let investor2_balance = Evercity::balance_everusd(&INVESTOR2);

    bond.mincap_deadline = 50000;
    assert_ok!(Evercity::bond_add_new(Origin::signed(acc), bondid, bond));
    <pallet_timestamp::Module<TestRuntime>>::set_timestamp(10_000);
    assert_ok!(Evercity::bond_release(Origin::signed(MASTER), bondid));
    let chain_bond_item = Evercity::get_bond(&bondid);
    assert_eq!(chain_bond_item.issued_amount, 0);

    // Buy two packages
    assert_ok!(Evercity::bond_unit_package_buy(
        Origin::signed(INVESTOR1),
        bondid,
        600
    ));
    <pallet_timestamp::Module<TestRuntime>>::set_timestamp(20_000);
    assert_ok!(Evercity::bond_unit_package_buy(
        Origin::signed(INVESTOR2),
        bondid,
        600
    ));

    let chain_bond_item = Evercity::get_bond(&bondid);
    assert_eq!(chain_bond_item.issued_amount, 1200);
    assert_eq!(chain_bond_item.bond_debit, 1200 * 4_000_000_000_000);
    assert_eq!(chain_bond_item.bond_debit, chain_bond_item.bond_credit);

    assert_ok!(Evercity::bond_set_auditor(
        Origin::signed(MASTER),
        bondid,
        AUDITOR
    ));

    // Activate bond
    <pallet_timestamp::Module<TestRuntime>>::set_timestamp(30000);
    assert_ok!(Evercity::bond_activate(Origin::signed(MASTER), bondid));
    let chain_bond_item = Evercity::get_bond(&bondid);

    assert_eq!(chain_bond_item.issued_amount, 1200);
    assert_eq!(chain_bond_item.bond_debit, 0);
    assert_eq!(chain_bond_item.bond_credit, 0);

    assert_eq!(Evercity::balance_everusd(&acc), 1200 * 4_000_000_000_000);

    assert_eq!(
        investor1_balance - Evercity::balance_everusd(&INVESTOR1),
        600 * 4_000_000_000_000
    );
    assert_eq!(
        investor2_balance - Evercity::balance_everusd(&INVESTOR2),
        600 * 4_000_000_000_000
    );
    // Try revoke
    assert_noop!(
        Evercity::bond_revoke(Origin::signed(acc), bondid),
        RuntimeError::BondStateNotPermitAction
    );
    // Try give back
    assert_noop!(
        Evercity::bond_unit_package_return(Origin::signed(INVESTOR1), bondid, 600),
        RuntimeError::BondStateNotPermitAction
    );
}

#[test]
fn bond_create_release_update() {
    let bond = get_test_bond();
    let bondid: BondId = "TEST".into();

    const ACCOUNT: u64 = 3;
    const MASTER: u64 = 1;
    const MANAGER: u64 = 8;
    new_test_ext().execute_with(|| {
        assert_ok!(Evercity::bond_add_new(
            Origin::signed(ACCOUNT),
            bondid,
            bond.inner.clone()
        ));
        let chain_bond_item = Evercity::get_bond(&bondid);
        assert_eq!(chain_bond_item.state, BondState::PREPARE);

        // set Manager
        assert_noop!(
            Evercity::bond_set_manager(Origin::signed(ACCOUNT), bondid, MANAGER),
            RuntimeError::AccountNotAuthorized
        );
        assert_ok!(Evercity::bond_set_manager(
            Origin::signed(MASTER),
            bondid,
            MANAGER
        ));
        // Manager can change bond_units_base_price
        let mut new_bond = bond.inner.clone();
        new_bond.bond_units_base_price = 100000;
        assert_ok!(Evercity::bond_update(
            Origin::signed(MANAGER),
            bondid,
            new_bond
        ));

        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(10_000);

        assert_ok!(Evercity::bond_release(Origin::signed(MASTER), bondid));
        let chain_bond_item = Evercity::get_bond(&bondid);
        assert_eq!(chain_bond_item.state, BondState::BOOKING);
        assert_eq!(chain_bond_item.booking_start_date, 10_000);
        assert_eq!(chain_bond_item.manager, MANAGER);
        assert_eq!(chain_bond_item.inner.bond_units_base_price, 100_000);
    });
}

#[test]
fn bond_activate_bond_and_withdraw_bondfund() {
    const ACCOUNT: u64 = 3;
    let bondid: BondId = "BOND1".into();

    new_test_ext().execute_with(|| {
        bond_grand_everusd();
        bond_activate(bondid, ACCOUNT, get_test_bond().inner);
        let chain_bond_item = Evercity::get_bond(&bondid);
        assert_eq!(chain_bond_item.state, BondState::ACTIVE);
        assert_eq!(chain_bond_item.active_start_date, 30000);
        assert_eq!(chain_bond_item.bond_debit, 0);
        assert_eq!(chain_bond_item.bond_credit, 0);

        assert_eq!(
            Evercity::balance_everusd(&ACCOUNT),
            1200 * 4_000_000_000_000
        );

        let chain_bond_item = Evercity::get_bond(&bondid);
        assert_eq!(chain_bond_item.bond_debit, 0);
        assert_eq!(
            Evercity::balance_everusd(&ACCOUNT),
            1200 * 4_000_000_000_000
        );
    });
}

#[test]
fn bond_buy_bond_units_time_passed_after_activation() {
    const ACCOUNT: u64 = 3;
    const INVESTOR1: u64 = 4;
    let bondid: BondId = "BOND1".into();

    new_test_ext().execute_with(|| {
        bond_grand_everusd();
        bond_activate(bondid, ACCOUNT, get_test_bond().inner);
        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(600_000);
        assert_ok!(Evercity::bond_unit_package_buy(
            Origin::signed(INVESTOR1),
            bondid,
            400
        ));

        let chain_bond_item = Evercity::get_bond(&bondid);
        assert_eq!(
            Evercity::balance_everusd(&ACCOUNT),
            1600 * 4_000_000_000_000
        ); // (600 + 600 + 400) * 4000
        assert_eq!(chain_bond_item.bond_debit, 0);
        assert_eq!(bond_current_period(&chain_bond_item, 600_000), 0);
    });
}

#[test]
fn bond_give_back_bondunit_package() {
    const ACCOUNT: u64 = 3;
    const MASTER: u64 = 1;

    const INVESTOR1: u64 = 4;
    const INVESTOR2: u64 = 6;

    let bondid: BondId = "BOND0".into();

    new_test_ext().execute_with(|| {
        assert_ok!(add_token(INVESTOR1, 6_000_000_000_000_000));
        assert_ok!(add_token(INVESTOR2, 6_000_000_000_000_000));

        let mut bond = get_test_bond().inner;
        bond.mincap_deadline = 50000;

        assert_ok!(Evercity::bond_add_new(
            Origin::signed(ACCOUNT),
            bondid,
            bond
        ));
        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(10000);
        assert_ok!(Evercity::bond_release(Origin::signed(MASTER), bondid));

        assert_ok!(Evercity::bond_unit_package_buy(
            Origin::signed(INVESTOR1),
            bondid,
            600
        ));
        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(20000);
        assert_ok!(Evercity::bond_unit_package_buy(
            Origin::signed(INVESTOR2),
            bondid,
            600
        ));

        let packages1 = Evercity::bond_packages(&bondid, &INVESTOR1);
        assert_eq!(packages1.len(), 1);
        assert_eq!(packages1[0].bond_units, 600);
        assert_ok!(Evercity::bond_unit_package_return(
            Origin::signed(INVESTOR1),
            bondid,
            600
        ));

        let packages1 = Evercity::bond_packages(&bondid, &INVESTOR1);
        assert_eq!(packages1.len(), 0);
        // you cannot give back part of the package
        assert_noop!(
            Evercity::bond_unit_package_return(Origin::signed(INVESTOR2), bondid, 100),
            RuntimeError::BondParamIncorrect
        );

        let packages2 = Evercity::bond_packages(&bondid, &INVESTOR2);
        assert_eq!(packages2.len(), 1);
    });
}

#[test]
fn bond_iter_periods() {
    const ACCOUNT: u64 = 3;
    let bondid: BondId = "BOND1".into();

    let mut ext = new_test_ext();
    ext.execute_with(|| {
        bond_grand_everusd();
        bond_activate(bondid, ACCOUNT, get_test_bond().inner);
        let chain_bond_item = Evercity::get_bond(&bondid);

        for _period in chain_bond_item.iter_periods() {
            //println!("{:?}", period);
        }
        let periods: Vec<_> = chain_bond_item.iter_periods().collect();
        assert_eq!(periods.len(), 13);
    });
    //let db = ext.offchain_db();
}

#[test]
fn bond_cancel_after_release() {
    const ACCOUNT: u64 = 3;
    const MASTER: u64 = 1;
    const INVESTOR1: u64 = 4;
    const INVESTOR2: u64 = 6;
    let bondid: BondId = "BOND1".into();

    new_test_ext().execute_with(|| {
        assert_ok!(add_token(INVESTOR1, 10_000_000_000_000_000));
        assert_ok!(add_token(INVESTOR2, 10_000_000_000_000_000));

        let mut bond = get_test_bond().inner;
        bond.mincap_deadline = 50000;
        assert_ok!(Evercity::bond_add_new(
            Origin::signed(ACCOUNT),
            bondid,
            bond
        ));
        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(10000);
        assert_ok!(Evercity::bond_release(Origin::signed(MASTER), bondid));

        // Buy three packages
        assert_ok!(Evercity::bond_unit_package_buy(
            Origin::signed(INVESTOR1),
            bondid,
            400
        ));
        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(20_000);
        assert_ok!(Evercity::bond_unit_package_buy(
            Origin::signed(INVESTOR2),
            bondid,
            200
        ));
        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(30_000);
        assert_ok!(Evercity::bond_unit_package_buy(
            Origin::signed(INVESTOR2),
            bondid,
            200
        ));

        let chain_bond_item = Evercity::get_bond(&bondid);
        assert_eq!(chain_bond_item.issued_amount, 800);
        assert_eq!(chain_bond_item.bond_debit, 800 * 4_000_000_000_000);
        assert_eq!(chain_bond_item.bond_debit, chain_bond_item.bond_credit);

        assert_eq!(
            Evercity::balance_everusd(&INVESTOR1),
            10_000_000_000_000_000 - 400 * 4_000_000_000_000
        );
        assert_eq!(
            Evercity::balance_everusd(&INVESTOR2),
            10_000_000_000_000_000 - 400 * 4_000_000_000_000
        );

        // Bond unit packages

        let packages1 = Evercity::bond_packages(&bondid, &INVESTOR1);
        let packages2 = Evercity::bond_packages(&bondid, &INVESTOR2);

        assert_eq!(packages1.len(), 1);
        assert_eq!(packages2.len(), 2);

        assert_eq!(packages1[0].bond_units, 400);
        assert_eq!(packages2[0].bond_units, 200);
        assert_eq!(packages2[0].bond_units, 200);

        assert_eq!(packages1[0].create_date, 10000);
        assert_eq!(packages2[0].create_date, 20000);
        assert_eq!(packages2[1].create_date, 30000);

        assert_eq!(packages1[0].acquisition, 0);
        assert_eq!(packages2[0].acquisition, 0);
        assert_eq!(packages2[1].acquisition, 0);

        // We raised up less than  bond_units_mincap_amount, so we should revoke the bond
        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(60000);
        assert_ok!(Evercity::bond_withdraw(Origin::signed(MASTER), bondid));
        let chain_bond_item = Evercity::get_bond(&bondid);

        assert_eq!(chain_bond_item.issued_amount, 0);
        assert_eq!(chain_bond_item.state, BondState::PREPARE);
        assert_eq!(chain_bond_item.bond_debit, 0);
        assert_eq!(chain_bond_item.bond_credit, 0);

        assert_eq!(
            Evercity::balance_everusd(&INVESTOR1),
            10_000_000_000_000_000
        );
        assert_eq!(
            Evercity::balance_everusd(&INVESTOR2),
            10_000_000_000_000_000
        );

        let packages1 = Evercity::bond_packages(&bondid, &INVESTOR1);
        let packages2 = Evercity::bond_packages(&bondid, &INVESTOR2);

        assert_eq!(packages1.len(), 0);
        assert_eq!(packages2.len(), 0);
    });
}
