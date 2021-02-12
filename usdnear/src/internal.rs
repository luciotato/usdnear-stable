use crate::*;

pub use crate::types::*;
pub use crate::utils::*;

pub fn assert_min_amount(amount: u128) {
    assert!(amount >= FIVE_NEAR, "minimun amount is 5");
}

/****************************/
/* general Internal methods */
/****************************/
impl UsdNearStableCoin {
    
    /// Asserts that the method was called by the owner.
    pub(crate) fn assert_owner_calling(&self) {
        assert!(&env::predecessor_account_id()==&self.owner_account_id,"Can only be called by the owner");
    }

    /// Asserts the contract is not busy between async calls
    pub(crate) fn assert_not_busy(&self) {
        assert!(!self.busy,"busy");
    }


    //applies current_price to a stNEAR amount to get a USD valuation
    pub(crate) fn stnear_to_usd(&self, stnear:u128) -> u128 {
        return (U256::from(stnear) * U256::from(self.current_stnear_price) / U256::from(NEAR)).as_u128();
    }

    //applies current_price to convert from USDNEAR to stNEAR 
    pub(crate) fn usdnear_to_stnear(&self, usdnear:u128) -> u128 {
        return (U256::from(usdnear) * U256::from(NEAR) / U256::from(self.current_stnear_price)).as_u128();
    }
    

    //internal fn MUST not panic
    //removes from FREE stNEAR pool when the user withdraws
    pub(crate) fn remove_amount_and_free_shares_preserve_share_price(
        &mut self,
        account_id: &AccountId,
        amount: u128,
    ) {
        if amount > 0 {
            let num_shares = self.free_shares_from_amount(amount);
            //burn shares in the user acc
            let acc = &mut self.internal_get_account(account_id);
            acc.free_shares = acc.free_shares.saturating_sub(num_shares);
            self.internal_update_account(account_id, &acc);
            // reduce stNEAR amount and burn total shares in the contract
            self.total_free_shares = self.total_free_shares.saturating_sub(num_shares);
            self.total_free_stnear = self.total_free_stnear.saturating_sub(amount);
        }
    }

    //internal fn MUST not panic
    //removes from FREE stNEAR pool when the user withdraws
    /*
    pub(crate) fn remove_amount_and_locked_shares_preserve_share_price(
        &mut self,
        account_id: &AccountId,
        amount: u128,
    ) {
        if amount > 0 {
            let num_shares = self.locked_shares_from_amount(amount);
            //burn shares in the user acc
            let acc = &mut self.internal_get_account(account_id);
            acc.locked_collateral_shares = acc.locked_collateral_shares.saturating_sub(num_shares);
            self.internal_update_account(account_id, &acc);
            // reduce stNEAR amount and burn total shares in the contract
            self.total_collateral_shares = self.total_collateral_shares.saturating_sub(num_shares);
            self.total_collateral_stnear = self.total_collateral_stnear.saturating_sub(amount);
        }
    }
    */

    //--------------------------------
    pub(crate) fn add_amount_and_free_shares_preserve_share_price(
        &mut self,
        account_id: AccountId,
        amount: u128,
    ) {
        if amount > 0 {
            let num_shares = self.free_shares_from_amount(amount);
            let account = &mut self.internal_get_account(&account_id);
            account.free_shares += num_shares;
            &self.internal_update_account(&account_id, &account);
            // Increasing the total amount of free shares.
            self.total_free_shares += num_shares;
            self.total_free_stnear += amount;
        }
    }
    
    /*
    pub(crate) fn add_amount_and_locked_shares_preserve_share_price(
        &mut self,
        account_id: AccountId,
        amount: u128,
    ) {
        if amount > 0 {
            let num_shares = self.locked_shares_from_amount(amount);
            let account = &mut self.internal_get_account(&account_id);
            account.locked_collateral_shares += num_shares;
            &self.internal_update_account(&account_id, &account);
            // Increasing the total amount of collateral shares.
            self.total_collateral_shares += num_shares;
            self.total_collateral_stnear += amount;
        }
    }
    */

    /// Returns the amount corresponding to the given number of usdnear-owed shares.
    pub(crate) fn amount_from_usdnear_shares(&self, num_shares: u128) -> u128 {
        return amount_from_shares(num_shares, self.total_usdnear, self.total_usdnear_shares);
    }
    /// Returns the number of shares corresponding to the given USDNEAR amount
    /// if the amount & the shares are incorporated, price remains the same
    pub(crate) fn usdnear_shares_from_amount(&self, usdnear_amount: u128) -> u128 {
        return shares_from_amount(usdnear_amount, self.total_usdnear, self.total_usdnear_shares);
    }

    /// Returns the amount corresponding to the given number of free shares.
    pub(crate) fn amount_from_free_shares(&self, num_shares: u128) -> u128 {
        return amount_from_shares(num_shares, self.total_free_stnear, self.total_free_shares);
    }
    /// Returns the number of shares corresponding to the given stnear amount at current share_price
    /// if the amount & the shares are incorporated, price remains the same
    pub(crate) fn free_shares_from_amount(&self, amount: u128) -> u128 {
        return shares_from_amount(amount, self.total_free_stnear, self.total_free_shares);
    }
    
    /// Returns the amount corresponding to the given number of collateral shares.
    pub(crate) fn amount_from_collateral_shares(&self, num_shares: u128) -> u128 {
        return amount_from_shares(num_shares, self.total_collateral_stnear, self.total_collateral_shares);
    }
    /// Returns the number of shares corresponding to the given stnear amount at current share_price
    /// if the amount & the shares are incorporated, price remains the same
    pub(crate) fn locked_shares_from_amount(&self, amount: u128) -> u128 {
        return shares_from_amount(amount, self.total_collateral_stnear, self.total_collateral_shares);
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
            self.b_accounts.insert(account_id, account); //insert_or_update
        }
    }

    /// Inner method to get usdnear balance for an account or 0
    pub(crate) fn get_usdnear_balance(&self, account_id: &String) -> u128 {
        return self.usdnear_balances.get(&account_id).unwrap_or_default();
    }
    /// Inner method to save the given usdnear balance account 
    pub(crate) fn set_usdnear_balance(&mut self, account_id: &String, balance: u128) {
        if balance==0 {
            self.usdnear_balances.remove(&account_id);    
        }
        else {
            self.usdnear_balances.insert(&account_id,&balance);
        }
    }

    pub(crate) fn usdnear_transfer(&mut self, sender_id: &AccountId, receiver_id: &AccountId, amount:u128) {
        let sender_balance = self.get_usdnear_balance(&sender_id);
        let receiver_balance = self.get_usdnear_balance(&receiver_id);
        //check sender balance
        assert!(sender_balance>=amount,"Not enough balance {}",sender_balance);
        //update balances
        self.set_usdnear_balance(&sender_id, sender_balance - amount);
        self.set_usdnear_balance(&receiver_id, receiver_balance + amount);
    }

}
