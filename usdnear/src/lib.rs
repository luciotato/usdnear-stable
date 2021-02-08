//! A smart contract that allows diversified staking, stNEAR and STBL farming
//! this contract include parts of core-contracts/lockup-contract & core-contracts/staking-pool

/********************************/
/* CONTRACT Self Identification */
/********************************/
// [NEP-129](https://github.com/nearprotocol/NEPs/pull/129)
// see also pub fn get_contract_info
const CONTRACT_NAME: &str = "USDNEAR-StableCoin";
const CONTRACT_VERSION: &str = "0.1.0";
const DEFAULT_WEB_APP_URL: &str = "https://www.narwallets.com/dapp/testnet/usdnear/";
const DEFAULT_AUDITOR_ACCOUNT_ID: &str = "auditors.near";

use near_sdk::{env, ext_contract, near_bindgen, AccountId};
use near_sdk::json_types::Base58PublicKey;
use near_sdk::collections::{PersistentMap,UnorderedMap};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

pub use crate::internal::*;
pub use crate::owner::*;
pub use crate::getters::*;
pub use crate::types::*;
pub use crate::utils::*;

pub mod gas;
pub mod types;
pub mod utils;
pub mod getters;
pub mod internal;
pub mod owner;

#[cfg(target = "wasm32")]
#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc = near_sdk::wee_alloc::WeeAlloc::INIT;

pub const NSLP_INTERNAL_ACCOUNT: &str = "..NSLP..";

#[ext_contract(ext_staking_pool)]
pub trait ExtstNEARContract {
    fn get_account_staked_balance(&self, account_id: AccountId) -> U128String;
}

// -----------------
// User BorrowingAccount Data
// -----------------
#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub struct BorrowingAccount {
    /// The amount of shares of the total staked collateral this user owns.
    /// Because the amount of stNEAR naturally increases with epoch rewards, each acc has an amount of "shares" to be converted to a stNEAR amount on demand
    /// shares * share_price = amount of stNEARs
    collateral_shares: u128,
    ///shares locked from borrowing
    locked_collateral_shares: u128, 
    ///amount of usdnear owed
    outstanding_loans_usdnear: u128, 
    //-- STBL
    // governance token earned
    pub stbl: u128,

}

/// User BorrowingAccount on this contract
impl Default for BorrowingAccount {
    fn default() -> Self {
        Self {
            usdnear: 0,
            collateral_shares: 0,
            locked_collateral_shares:0,
            outstanding_loans_usdnear:0,
            stbl: 0,
        }
    }
}

impl BorrowingAccount {
    /// when the account.is_empty() it will be removed
    fn is_empty(&self) -> bool {
        return self.collateral_shares == 0
            && self.stbl == 0
            && self.locked_collateral_shares == 0
            && self.oustanding_loans_usdnear == 0
            ;
    }
}


//------------------------
//  Main Contract State --
//------------------------
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct UsdNearStableCoin {
    /// Owner's account ID (it will be a DAO on phase II)
    pub owner_account_id: String,

    /// There's a min balance you must mantain to backup storage usage
    /// can be adjusted down by keeping the required NEAR in the developers or operator account
    pub min_account_balance: u128,

    // Configurable info for [NEP-129](https://github.com/nearprotocol/NEPs/pull/129)
    pub web_app_url: Option<String>, 
    pub auditor_account_id: Option<String>,

    /// This amount increments with minting (borrowing) and decrements with burns (repayment)
    pub total_usdnear: u128,

    /// how many "collateral shares" were minted. Everytime someone desposits stNEAR (collateral) they get collateral_shares
    // the buy share price is computed so if she "sells" the shares on that moment she recovers the same stNEAR amount
    // when someone withdraws stNEAR they burn X shares at current price to recoup Y stNEAR
    pub total_collateral_shares: u128,

    /// STBL is the governance token. Total stbl minted
    pub total_stbl: u128,

    /// if the price oracle is delayed or the market is too unstable, borrowing can be paused
    pub borrowing_paused: bool,

    //user's usdnear balances. Separated so a user receiving/having only USDNEAR uses a low amount of storage
    pub usdnear_balances: PersistentMap<String, u128>,

    //user's borrowing accounts. hold collateral shares and outstanding loans
    pub b_accounts: UnorderedMap<String, BorrowingAccount>,

    ///annual percentage rate for outstandig loans
    pub usdnear_apr_basis_points: u16, //250 => 2.5%

    /// Operator account ID (who's in charge of the price oracle)
    pub operator_account_id: String,
    /// operator_fee_basis_points
    pub operator_fee_basis_points: u16,

    /// Treasury account ID (it will be controlled by a DAO on phase II)
    pub treasury_account_id: String,
    /// treasury cut on SHKASH Sell cut (25% default)
    pub treasury_fee_basis_points: u16,
}

impl Default for UsdNearStableCoin {
    fn default() -> Self {
        env::panic(b"The contract is not initialized.");
    }
}

#[near_bindgen]
impl UsdNearStableCoin {
    /* NOTE
    This contract implements several traits

    1. deposit-trait [NEP-xxx]: this contract implements: deposit, get_account_total_balance, get_account_available_balance, withdraw, withdraw_all
       A [NEP-xxx] contract creates an account on deposit and allows you to withdraw later under certain conditions. Deletes the account on withdraw_all

    4. multitoken (TODO) [NEP-xxx]: this contract implements: deposit(tok), get_token_balance(tok), withdraw_token(tok), tranfer_token(tok), transfer_token_to_contract(tok)
       A [NEP-xxx] manages multiple tokens

    */

    /// Requires 25 TGas (1 * BASE_GAS)
    ///
    /// Initializes UsdNearStableCoin contract.
    /// - `owner_account_id` - the account ID of the owner.  Only this account can call owner's methods on this contract.
    #[init]
    pub fn new(
        owner_account_id: AccountId,
        treasury_account_id: AccountId,
        operator_account_id: AccountId,
    ) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized");

        return Self {
            owner_account_id,
            operator_account_id,
            treasury_account_id,
            min_account_balance: ONE_NEAR,
            web_app_url: Some(String::from(DEFAULT_WEB_APP_URL)),
            auditor_account_id: Some(String::from(DEFAULT_AUDITOR_ACCOUNT_ID)),
            operator_fee_basis_points: DEFAULT_OPERATOR_REWARDS_FEE_BASIS_POINTS,
            operator_swap_cut_basis_points: DEFAULT_OPERATOR_SWAP_CUT_BASIS_POINTS,
            treasury_fee_basis_points: DEFAULT_TREASURY_SWAP_CUT_BASIS_POINTS,
            borrowing_paused: false, 
            total_usdnear: 0,
            total_collateral_stnear: 0,
            total_collateral_shares: 0,
            total_stbl: 0,
            usdnear_balances: PersistentMap::new("U".into()),
            b_accounts: UnorderedMap::new("A".into()),
            usdnear_apr_basis_points: 250,   //2.5%
        };
    }
  

}
