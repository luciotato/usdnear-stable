use crate::*;
use near_sdk::{near_bindgen, Balance};

pub use crate::types::*;
pub use crate::utils::*;

/****************************/
/* general Internal methods */
/****************************/
impl UsdNearStableCoin {
    /// Asserts that the method was called by the owner.
    pub fn assert_owner_calling(&self) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.owner_account_id,
            "Can only be called by the owner"
        )
    }
}

pub fn assert_min_amount(amount: u128) {
    assert!(amount >= FIVE_NEAR, "minimun amount is 5N");
}

/***************************************/
/* Internal methods staking-pool trait */
/***************************************/
#[near_bindgen]
impl UsdNearStableCoin {

    //------------------------------
    pub(crate) fn internal_withdraw_stnear(&mut self, stnear_amount_requested: u128) {
        
        let account_id = env::predecessor_account_id();
        let acc = self.internal_get_account(&account_id);

        let total_stnear = self.amount_from_collateral_shares(acc.collateral_shares);
        let locked_stnear = self.amount_from_collateral_shares(acc.locked_collateral_shares);
        let stnear_available = total_stnear - locked_stnear;

        assert!(
            stnear_available >= stnear_amount_requested,
            "Not enough stNEAR balance to withdraw the requested amount. You have stNEAR {} total, {} locked and {} available", 
            total_stnear, locked_stnear, stnear_available 
        );

        let amount_to_transfer  = 
        if stnear_available - stnear_amount_requested < ONE_NEAR_CENT/2  //small yotctos remain, withdraw all
            { stnear_available } 
        else  { stnear_amount_requested };

        //launch async to trasnfer stNEAR from this contract to the user
        ext_meta_pool::ft_transfer(
            account_id.clone(),
            amount_to_transfer.into(),
            String::from(""), //memo
            //------------
            &META_POOL_STNEAR_CONTRACT,
            NO_DEPOSIT,
            gas::TRANSFER_STNEAR,
        )
        .then(ext_self_owner::after_transfer_stnear_to_user( //after transfer continue here
            account_id,
            amount_to_transfer,
            //------------
            &env::current_account_id(),
            NO_DEPOSIT,
            gas::AFTER_TRANSFER_STNEAR,
        ));
    }

    //prev fn continues here
    /// Called after transfer stNear to the user
    pub fn after_transfer_stnear_to_user(
        &mut self,
        account_id: String,
        amount: u128,
    ) {
        assert_callback_calling();

        //if we reached here, the stNEAR trasnfer was successful

        //remove stNear from contract and account
        let shares = self.collateral_shares_from_amount(amount);

        self.total_collateral_stnear -= amount;
        self.total_collateral_shares -= shares;//burn total shares

        let mut acc = self.internal_get_account(&account_id);
        acc.collateral_shares -= shares; //burn acc shares
        self.internal_update_account(&account_id, &acc);
    }


    //--------------------------------
    pub(crate) fn add_amount_and_shares_preserve_share_price(
        &mut self,
        account_id: AccountId,
        amount: u128,
    ) {
        if amount > 0 {
            let num_shares = self.collateral_shares_from_amount(amount);
            if num_shares > 0 {
                let account = &mut self.internal_get_account(&account_id);
                account.collateral_shares += num_shares;
                &self.internal_update_account(&account_id, &account);
                // Increasing the total amount of "stake" shares.
                self.total_collateral_shares += num_shares;
                self.total_collateral_stnear += amount;
            }
        }
    }

    /// Returns the number of shares corresponding to the given stnear amount at current share_price
    /// if the amount & the shares are incorporated, price remains the same
    pub(crate) fn collateral_shares_from_amount(&self, amount: Balance) -> u128 {
        return shares_from_amount(amount, self.total_collateral_stnear, self.total_collateral_shares);
    }

    /// Returns the amount corresponding to the given number of "stake" shares.
    pub(crate) fn amount_from_collateral_shares(&self, num_shares: u128) -> u128 {
        return amount_from_shares(num_shares, self.total_collateral_stnear, self.total_collateral_shares);
    }

    /// Inner method to get the given account or a new default value account.
    pub(crate) fn internal_get_account(&self, account_id: &String) -> BorrowingAccount {
        self.b_accounts.get(account_id).unwrap_or_default()
    }

    /// Inner method to save the given account for a given account ID.
    /// If the account balances are 0, the account is deleted instead to release storage.
    pub(crate) fn internal_update_account(&mut self, account_id: &String, account: &BorrowingAccount) {
        if account.is_empty() {
            self.b_accounts.remove(account_id); //delete
        } else {
            self.b_accounts.insert(account_id, &account); //insert_or_update
        }
    }

}
