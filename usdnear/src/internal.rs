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

    //applies current_price to convert from USDNEAR to stNEAR (collateral)
    pub(crate) fn usdnear_to_stnear(&self, usdnear:u128) -> u128 {
        return (U256::from(usdnear) * U256::from(NEAR) / U256::from(self.current_stnear_price)).as_u128();
    }
    

    //internal fn MUST not panic
    pub(crate) fn remove_amount_and_shares_preserve_share_price(
        &mut self,
        account_id: &AccountId,
        amount: u128,
    ) {
        if amount > 0 {
            let num_shares = self.collateral_shares_from_amount(amount);
            //burn shares in the user acc
            let acc = &mut self.internal_get_account(account_id);
            acc.collateral_shares = acc.collateral_shares.saturating_sub(num_shares);
            self.internal_update_account(account_id, &acc);
            // reduce stNEAR amount and burn total shares in the contract
            self.total_collateral_shares = self.total_collateral_shares.saturating_sub(num_shares);
            self.total_collateral_stnear = self.total_collateral_stnear.saturating_sub(amount);
        }
    }

    //--------------------------------
    pub(crate) fn add_amount_and_shares_preserve_share_price(
        &mut self,
        account_id: AccountId,
        amount: u128,
    ) {
        if amount > 0 {
            let num_shares = self.collateral_shares_from_amount(amount);
            let account = &mut self.internal_get_account(&account_id);
            account.collateral_shares += num_shares;
            &self.internal_update_account(&account_id, &account);
            // Increasing the total amount of collateral shares.
            self.total_collateral_shares += num_shares;
            self.total_collateral_stnear += amount;
        }
    }

    /// Returns the number of shares corresponding to the given stnear amount at current share_price
    /// if the amount & the shares are incorporated, price remains the same
    pub(crate) fn collateral_shares_from_amount(&self, amount: u128) -> u128 {
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
            self.b_accounts.insert(account_id, account); //insert_or_update
        }
    }

    pub(crate) fn usdnear_transfer(&mut self, sender_id: &AccountId, receiver_id: &AccountId, amount:u128) {
        let sender_balance = self.usdnear_balances.get(&sender_id).unwrap_or_default();
        let receiver_balance = self.usdnear_balances.get(&receiver_id).unwrap_or_default();
        //check sender balance
        assert!(sender_balance>=amount,"Not enough balance {}",sender_balance);
        //update balances
        self.usdnear_balances.insert(&sender_id, &(sender_balance - amount));
        self.usdnear_balances.insert(&receiver_id, &(receiver_balance + amount));
    }

}
