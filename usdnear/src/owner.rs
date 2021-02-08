use crate::*;
use near_sdk::{near_bindgen, Promise, PublicKey};

#[near_bindgen]
impl UsdNearStableCoin {
    /// OWNER'S METHOD
    /// Adds full access key with the given public key to the account once the contract is empty
    /// (has no b_accounts)
    pub fn add_full_access_key(&mut self, new_public_key: Base58PublicKey) -> Promise {
        
        self.assert_owner_calling();

        assert!(self.b_accounts.len() == 0, "contract still has b_accounts");

        env::log(b"Adding a full access key");

        let new_public_key: PublicKey = new_public_key.into();

        Promise::new(env::current_account_id()).add_full_access_key(new_public_key)
    }

    /// Owner's method.
    /// Pauses borrowing
    pub fn pause_borrowing(&mut self) {
        self.assert_owner_calling();
        assert!(!self.borrowing_paused, "borrowing is already paused");
        self.borrowing_paused = true;
    }

}
