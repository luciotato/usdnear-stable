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
use near_sdk::collections::{UnorderedMap};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

pub use crate::internal::*;
pub use crate::owner::*;
pub use crate::types::*;
pub use crate::utils::*;

pub mod gas;
pub mod types;
pub mod utils;
pub mod internal;
pub mod owner;
pub mod funtoken;
pub mod persistent_map;

pub use persistent_map::*;

#[cfg(target = "wasm32")]
#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc = near_sdk::wee_alloc::WeeAlloc::INIT;

pub const NSLP_INTERNAL_ACCOUNT: &str = "..NSLP..";

#[ext_contract(ext_meta_pool)]
pub trait ExtMetaPoolContract {
    fn get_account_total_balance(&self, account_id: AccountId) -> U128String;
    fn ft_transfer(receiver_id: AccountId, amount: U128String, memo:Option<String>);
}

// callbacks here defined as traits to make it easy to create the promise
#[ext_contract(ext_self_callback)]
pub trait SelfCallbacks {

    fn after_transfer_stnear_to_user(
        &self,
        account_id: String,
        amount: U128String,
    );

    fn after_transfer_stnear_plus_fee_to_liquidator(
        &self,
        loan_account_id:AccountId,
        usdnear_repay:u128,
        liquidator_id:AccountId,
        stnear_to_receive:u128
    );

    fn after_ft_on_transfer_usdnear(&mut self, sender_id:AccountId, receiver_id: AccountId, amount: U128String, #[callback] unused_tokens: U128String);

    fn after_get_meta_contract_stnear_total_balance(
        &mut self,
        #[callback] total_balance: U128String,
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
    // governance token - TODO
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

    fn required_collateral_stnear(&self, main:&UsdNearStableCoin) -> u128 {
        if self.outstanding_loans_usdnear==0 {return 0}; 
        let required_collateral_usdnear = apply_pct(main.collateral_basis_points, self.outstanding_loans_usdnear);
        return main.usdnear_to_stnear(required_collateral_usdnear);
    }

    fn locked_collateral_stnear(&self, main:&UsdNearStableCoin) -> u128 {
        if self.outstanding_loans_usdnear==0 {return 0}; 
        let required_collateral_stnear = self.required_collateral_stnear(main);
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
    pub usdnear_apr_basis_points: u32, //250 => 2.5%
    pub epochs_per_year: u32, //365*2 epochs per year in NEAR

    ///liquidation fee. % the liquidator earns to restore overcollateralization
    pub liquidaton_fee_basis_points: u16, // default 1000 => 10%

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

    //locked while transferring via promises & callbacks
    pub busy: bool,

    pub last_rewards_epoch_height: EpochHeight,

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
            usdnear_apr_basis_points: 250,   //2.5%
            epochs_per_year: 365*2, 
            liquidaton_fee_basis_points: 1000, //10%
            min_account_balance: NEAR,
            web_app_url: Some(String::from(DEFAULT_WEB_APP_URL)),
            auditor_account_id: Some(String::from(DEFAULT_AUDITOR_ACCOUNT_ID)),
            operator_fee_basis_points: 3000, //30% from 2.5%
            treasury_fee_basis_points: 7000, //70% from 2.5%
            borrowing_paused: false,  
            total_usdnear: 0,
            total_collateral_stnear: 0,
            total_collateral_shares: 0,
            total_stbl: 0,
            usdnear_balances: PersistentMap::new("U".into()),
            b_accounts: UnorderedMap::new("A".into()),
            busy: false,
            last_rewards_epoch_height:0,
        };
    }

    /// ---Indirect DEPOSIT/ADD stNEAR COLLATERAL--- (stNEAR is a NEP-141 fungible token standard)
    /// To "deposit some stNEAR"/"add collateral" the web app must call META_POOL_STNEAR_CONTRACT.ft_transfer_call("usdnear.stable.testnet", [amount])
    /// the amount is transferred and then the META_POOL_STNEAR_CONTRACT will call this fn ft_on_transfer
    pub fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128String, _msg: String ) -> u128 { 
        //verify this is a callback from META_POOL_STNEAR_CONTRACT
        assert_eq!(env::predecessor_account_id(), META_POOL_STNEAR_CONTRACT);
        //register the stNEAR into our internal accounting for the sender
        self.add_amount_and_shares_preserve_share_price(sender_id, amount.0);
        //all stNEAR used
        return 0;
    }

    /// Withdraws collateral(stNEAR) from this contract to the user's META_POOL_STNEAR_CONTRACT account
    pub fn withdraw_stnear(&mut self, amount: U128String) {
        
        self.assert_not_busy();

        let account_id = env::predecessor_account_id();
        let acc = self.internal_get_account(&account_id);

        let total_stnear = self.amount_from_collateral_shares(acc.collateral_shares);
        let locked_stnear = acc.locked_collateral_stnear(self);
        let stnear_available = total_stnear.saturating_sub(locked_stnear);

        assert!(
            stnear_available >= amount.0,
            "Not enough stNEAR balance to withdraw the requested amount. You have stNEAR {} total, {} locked and {} available", 
            total_stnear, locked_stnear, stnear_available 
        );

        let amount_to_transfer  = 
            if stnear_available - amount.0 < ONE_NEAR_CENT/2  //small yotctos remain, withdraw all
                { stnear_available } 
            else 
                { amount.0 };

        //mark as busy - block reentry
        self.busy = true;

        //launch async to trasnfer stNEAR from this contract to the user
        ext_meta_pool::ft_transfer(
            account_id.clone(),
            amount_to_transfer.into(),
            None, //memo
            //------------
            &META_POOL_STNEAR_CONTRACT,
            NO_DEPOSIT,
            gas::TRANSFER_STNEAR,
        )
        .then(ext_self_callback::after_transfer_stnear_to_user( //after transfer callback here
            account_id,
            amount_to_transfer.into(),
            //------------
            &env::current_account_id(),
            NO_DEPOSIT,
            gas::AFTER_TRANSFER_STNEAR,
        ));
    }
    //prev fn continues here
    // Called after transfer stNear to the user
    //must not panic
    pub fn after_transfer_stnear_to_user(
        &mut self,
        account_id: String,
        amount: U128String,
    ) {
        assert_callback_calling();
        self.busy= false;
        //debug!("after_transfer {} {} {}",is_promise_success(),account_id,amount.0);
        if is_promise_success() {
            //the stNEAR withdrawal was successful
            self.remove_amount_and_shares_preserve_share_price(&account_id,amount.0);
        }
    }


    pub fn convert_usdnear(&mut self, usdnear_to_convert:U128String){
        // get usdnear balance for this user
        let usdnear_balance = self.usdnear_balances.get(&env::predecessor_account_id()).unwrap_or_default();
        // can't use what they don't have
        assert!(usdnear_balance>=usdnear_to_convert.0,"Noy enough balance, you have USDNEAR {}",usdnear_balance);
        //compute stNEAR amount
        let stnear = self.usdnear_to_stnear(usdnear_to_convert.0);
        //get account
        let mut acc = self.internal_get_account(&env::predecessor_account_id());
        // do the user owe usdnear?
        assert!(acc.outstanding_loans_usdnear>0,"You owe no USDNEAR");
        // max to repay is what they owe
        let to_repay = if usdnear_to_convert.0 > acc.outstanding_loans_usdnear {acc.outstanding_loans_usdnear} else {usdnear_to_convert.0};
        // repay & save acc
        acc.outstanding_loans_usdnear-=to_repay;
        self.internal_update_account(&env::predecessor_account_id(), &acc);
        // burn usdnear for the user & the contract
        self.usdnear_balances.insert(&env::predecessor_account_id(), &(usdnear_balance-to_repay));
        self.total_usdnear-=to_repay;

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

    /// if loan_account_id collateral ratio is below self.min_collateral_basis_points
    /// predecesor_account_id can try to liquidate X amount
    /// in order to move collateral ratio back to self.collateral_basis_points
    /// 
    pub fn liquidate(&mut self, loan_account_id:String, max_usdnear_buy:U128String) {

        self.assert_not_busy();

        assert!(max_usdnear_buy.0 >= TEN_NEAR, "minimun amount to buy is USDNEAR 10");

        let liquidator_id = env::predecessor_account_id();

        //liquidator must have a borrowingAccount here, with a min stNEAR balance
        let mut liquidator_acc = self.internal_get_account(&liquidator_id);
        assert!(self.amount_from_collateral_shares(liquidator_acc.collateral_shares) >= MIN_STNEAR_BALANCE_FOR_LIQUIDATORS,
            "To be a liquidator you need to have a borrowing account with at least stNEAR {}",MIN_STNEAR_BALANCE_FOR_LIQUIDATORS);

        //get loan account 
        let mut loan_acc = self.internal_get_account(&loan_account_id);
        // do the loan_acc owe usdnear?
        assert!(loan_acc.outstanding_loans_usdnear>0,"no USDNEAR owed");
        // check collateralization
        let rate = loan_acc.get_current_collateralization_ratio(self);
        assert!(rate < self.min_collateral_basis_points, "coll.rate.BP is {}. Can't liquidate",rate);
        // compute usdnear to repay in order to to restore collatellar rate
        let all_collateral_stnear = self.amount_from_collateral_shares(loan_acc.collateral_shares);
        let required_collateral_usd = apply_pct(self.collateral_basis_points, loan_acc.outstanding_loans_usdnear);
        let valued_collateral_usd = loan_acc.valued_collateral_usd(self);
        let liq_fee_plus_100:u32 = 10000+self.liquidaton_fee_basis_points as u32;
        //cross-check, shouldn't happen at this point
        assert!(valued_collateral_usd < required_collateral_usd, "ERR: valued.collat {} >= req.coll {}",valued_collateral_usd,required_collateral_usd);
        let max_usdnear_repay: u128;
        if valued_collateral_usd < loan_acc.outstanding_loans_usdnear { 
            //catasthrophic. underwater loan. It's the responsibility of the liquidator to check this condition before this call
            //at this point we accept the liquidation even if at face value is not benefical to the liquidator
            max_usdnear_repay = loan_acc.outstanding_loans_usdnear;
        }
        else {
            //some room for a liquidation fee
            //compute exact usdnear amount 
            max_usdnear_repay = (U256::from(required_collateral_usd - valued_collateral_usd) * U256::from(10000) / 
                    U256::from(self.collateral_basis_points - liq_fee_plus_100)).as_u128();
        }        

        //the amount to repay is limited to the amount the liquidator indicated as max
        let usdnear_repay = std::cmp::min(max_usdnear_repay, max_usdnear_buy.0);

        // get liquidator's usdnear balance
        let liquidator_usdnear_balance = self.usdnear_balances.get(&liquidator_id).unwrap_or_default();
        assert!(liquidator_usdnear_balance>=usdnear_repay,"not enough USDNEAR to repay loan. you need {}",usdnear_repay);

        //ok, the liquidation can proceed

        //from the liquidator, take usdnear amount, use it to repay loan
        self.usdnear_balances.insert(&liquidator_id, &(liquidator_usdnear_balance-usdnear_repay));
        // repay loan with liquidator's usdnear (burn usdnear)
        loan_acc.outstanding_loans_usdnear-=usdnear_repay;
        self.total_usdnear-=usdnear_repay;

        //stnear_to_receive should be usdnear*(1+fee%) worth of stnear, with a hard limit set at all_collateral_stnear
        let stnear_to_receive = std::cmp::min(all_collateral_stnear, self.usdnear_to_stnear(apply_pct(liq_fee_plus_100, usdnear_repay)));
        let collateral_shares_plus_fee = std::cmp::min(loan_acc.collateral_shares, self.collateral_shares_from_amount(stnear_to_receive));

        // move collateral stnear from loan_acc.collateral to liquidator
        loan_acc.collateral_shares-=collateral_shares_plus_fee;
        liquidator_acc.collateral_shares+=collateral_shares_plus_fee;

        // save loan acc
        self.internal_update_account(&loan_account_id, &loan_acc);
        // save liquidator acc
        self.internal_update_account(&liquidator_id, &liquidator_acc);

    }


}
