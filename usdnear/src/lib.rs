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
use near_sdk::collections::{UnorderedMap,LookupSet};
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
pub mod persistent_map;

pub use persistent_map::*;

#[cfg(target = "wasm32")]
#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc = near_sdk::wee_alloc::WeeAlloc::INIT;

pub const NSLP_INTERNAL_ACCOUNT: &str = "..NSLP..";

#[ext_contract(ext_meta_pool)]
pub trait ExtMetaPoolContract {
    fn get_account_staked_balance(&self, account_id: AccountId) -> U128String;
    fn ft_transfer(account_id: AccountId, amount_to_transfer: U128String, memo:String);
}

// callbacks here defined as traits to make it easy to create the promise
#[ext_contract(ext_self_owner)]
pub trait OwnerCallbacks {
    fn after_transfer_stnear_to_user(
        &mut self,
        account_id: String,
        amount: u128,
    );
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
            collateral_shares: 0,
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
            && self.outstanding_loans_usdnear == 0
            ;
    }

    fn valued_collateral_usd(&self, main:&UsdNearStableCoin) -> u128 {
        let actual_collateral_stnear = main.amount_from_collateral_shares(self.collateral_shares);
        return main.stnear_to_usd(actual_collateral_stnear);
    }

    fn locked_collateral_stnear(&self, main:&UsdNearStableCoin) -> u128 {
        if self.outstanding_loans_usdnear==0 {return 0}; 
        let required_collateral_usdnear = apply_pct(main.collateral_basis_points, self.outstanding_loans_usdnear);
        let required_collateral_stnear = main.usdnear_to_stnear(required_collateral_usdnear);
        let actual_collateral_stnear = main.amount_from_collateral_shares(self.collateral_shares);
        return if required_collateral_stnear<actual_collateral_stnear {required_collateral_stnear} else {actual_collateral_stnear};
    }

    //max usdnear for this acc, according to collateral, price and collateral_basis_points
    fn max_usdnear(&self, main:&UsdNearStableCoin) -> u128 {
        let valued_collateral_usd = self.valued_collateral_usd(main);
        return (U256::from(valued_collateral_usd) * U256::from(10000) / U256::from(main.collateral_basis_points)).as_u128();
    }

    fn get_current_credit_limit(&self, main:&UsdNearStableCoin) -> u128 {
        let max_usdnear = self.max_usdnear(main);
        return max_usdnear.saturating_sub(self.outstanding_loans_usdnear);
    }

    /// returns basis points
    /// if collateral ratio >999%, returns 999%
    fn get_current_collateralization_ratio(&self, main:&UsdNearStableCoin) -> u32 {
        const MAX:u32 = 999*PERCENT_BP;
        if self.outstanding_loans_usdnear==0 {return MAX}; 
        let valued_collateral = self.valued_collateral_usd(main);
        let ratio = (U256::from(valued_collateral) * U256::from(10000) / U256::from(self.outstanding_loans_usdnear)).as_u128();
        if ratio>MAX as u128 {return MAX}; 
        return ratio as u32;
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

    /// updated by external oracle
    pub current_stnear_price: u128,

    /// collateral % (default 200%)
    pub collateral_basis_points: u32, 
    
    /// liquidation collateral % (default 150%)
    /// collateral % when liquidation is opened
    pub min_collateral_basis_points: u32, 

    /// This amount increments with minting (borrowing) and decrements with burns (repayment)
    pub total_usdnear: u128,

    /// This amount increments with users depositing stNEAR and decrements with users withdrawing stNEAR
    /// This amouns also is incremented when the staking rewards are collected every epoch
    pub total_collateral_stnear: u128,
    /// how many "collateral shares" were minted. Everytime someone desposits stNEAR (collateral) they get collateral_shares
    /// the buy share price is computed so if they "sells" the shares on that moment they recover the same stNEAR amount
    /// when someone withdraws stNEAR they burn X shares at current price to recoup Y stNEAR
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

    /// There's a min balance you must mantain to backup storage usage
    /// can be adjusted down by keeping the required NEAR in the developers or operator account
    pub min_account_balance: u128,

    // Configurable info for [NEP-129](https://github.com/nearprotocol/NEPs/pull/129)
    pub web_app_url: Option<String>, 
    pub auditor_account_id: Option<String>,

    //locked accounts while transferring via promises & callbacks
    busy_accounts: LookupSet<String>, 

}

impl Default for UsdNearStableCoin {
    fn default() -> Self {
        env::panic(b"The contract is not initialized.");
    }
}

#[near_bindgen]
impl UsdNearStableCoin {
    /// Initializes UsdNearStableCoin contract.
    /// - `owner_account_id` - the account ID of the owner.  Only this account can call owner's methods on this contract.
    #[init]
    pub fn new(
        owner_account_id: AccountId,
        treasury_account_id: AccountId,
        operator_account_id: AccountId,
        current_stnear_price: U128String,
    ) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized");

        return Self {
            owner_account_id,
            operator_account_id,
            treasury_account_id,
            current_stnear_price: current_stnear_price.0,
            collateral_basis_points: 200*PERCENT_BP,
            min_collateral_basis_points: 150*PERCENT_BP,
            min_account_balance: NEAR,
            web_app_url: Some(String::from(DEFAULT_WEB_APP_URL)),
            auditor_account_id: Some(String::from(DEFAULT_AUDITOR_ACCOUNT_ID)),
            usdnear_apr_basis_points: 250,   //2.5%
            operator_fee_basis_points: 3000, //30% from 2.5%
            treasury_fee_basis_points: 7000, //70% from 2.5%
            borrowing_paused: false,  
            total_usdnear: 0,
            total_collateral_stnear: 0,
            total_collateral_shares: 0,
            total_stbl: 0,
            usdnear_balances: PersistentMap::new("U".into()),
            b_accounts: UnorderedMap::new("A".into()),
            busy_accounts: LookupSet::new("B".into()),
        };
    }

    //applies current_price to a stNEAR amount to get a USD valuation
    fn stnear_to_usd(&self, stnear:u128) -> u128 {
        return (U256::from(stnear) * U256::from(self.current_stnear_price) / U256::from(NEAR)).as_u128();
    }

    //applies current_price to convert from USDNEAR to stNEAR (collateral)
    fn usdnear_to_stnear(&self, usdnear:u128) -> u128 {
        return (U256::from(usdnear) * U256::from(NEAR) / U256::from(self.current_stnear_price)).as_u128();
    }
    
    /// ---Indirect DEPOSIT/ADD COLLATERAL--- (NEP-141 fungible token standard)
    /// To "deposit some stNEAR"/"add collateral" the web app must call META_POOL_STNEAR_CONTRACT.ft_transfer_call("usdnear.stable.testnet", [amount])
    /// the amount is transferred and then the META_POOL_STNEAR_CONTRACT will call this fn ft_on_transfer
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128String,
        _msg: String,
    ) -> u128 { 
        //verify this is a callback from META_POOL_STNEAR_CONTRACT
        assert_eq!(env::predecessor_account_id(), META_POOL_STNEAR_CONTRACT);
        //register the stNEAR into our internal accounting for the sender
        self.add_amount_and_shares_preserve_share_price(sender_id, amount.0);
        //all stNEAR used
        return 0;
    }

    /// Withdraws collateral(stNEAR) from this contract to the user's META_POOL_STNEAR_CONTRACT account
    pub fn withdraw_stnear(&mut self, amount: U128String) {
        self.internal_withdraw_stnear(amount.into());
    }

    pub fn take_loan(&mut self, usdnear_amount:U128String) {
        assert!(usdnear_amount.0>=1*NEAR,"min loan is 1 USDNEAR");
        //get account
        let mut acc = self.internal_get_account(&env::predecessor_account_id());
        //get current creditr limit
        let limit = acc.get_current_credit_limit(&self);
        assert!(usdnear_amount.0<=limit,"You can only take USDNEAR {} as loan. Add more collateral to extend your credit",limit);
        //take loan & update acc
        acc.outstanding_loans_usdnear+=usdnear_amount.0;
        self.internal_update_account(&env::predecessor_account_id(), &acc);
        //get current balance
        let usdnear_balance = self.usdnear_balances.get(&env::predecessor_account_id()).unwrap_or_default();
        //add newly minted usdenars
        self.usdnear_balances.insert(&env::predecessor_account_id(), &(usdnear_balance+usdnear_amount.0));
        //add also to contract total
        self.total_usdnear+=usdnear_amount.0;
    }

    pub fn repay_loan(&mut self, usdnear_amount:U128String) {
        //get account
        let mut acc = self.internal_get_account(&env::predecessor_account_id());
        // do the user owe usdnear?
        assert!(acc.outstanding_loans_usdnear>0,"You owe no USDNEAR");
        // max to repay is what they owe
        let to_repay = if usdnear_amount.0 > acc.outstanding_loans_usdnear {acc.outstanding_loans_usdnear} else {usdnear_amount.0};
        // get usdnear balance for this user
        let usdnear_balance = self.usdnear_balances.get(&env::predecessor_account_id()).unwrap_or_default();
        // can't use what they don't have
        assert!(usdnear_balance>=to_repay,"You have USDNEAR {}. You can not repay {}",usdnear_balance,to_repay);
        // repay & save acc
        acc.outstanding_loans_usdnear-=to_repay;
        self.internal_update_account(&env::predecessor_account_id(), &acc);
        // burn usdnear for the user & the contract
        self.usdnear_balances.insert(&env::predecessor_account_id(), &(usdnear_balance-to_repay));
        self.total_usdnear-=to_repay;
    }

}
