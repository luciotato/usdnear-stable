//
// NEP-141 Fungible Token Standard implementation
//

pub const USDNEAR:&str="USDNEAR";

use near_sdk::{near_bindgen};
use crate::*;

// callbacks here defined as traits to make it easy to create the promise
#[ext_contract(ext_ft_receiver)]
pub trait FunTokReceiver {
    //NEP-141 single fun token for the default token USDNEAR
    fn ft_on_transfer(&mut self,sender_id: AccountId,amount: U128String, _msg: String); 
}

pub fn assert_one_yocto(){
    assert!(env::attached_deposit()==1,"fn requires to attach 1 yoctoNEAR")
}

#[near_bindgen]
impl UsdNearStableCoin {

    /// Transfer `amount` of tokens from the caller (`predecessor_id`) to `receiver_id`.
    /// Requirements:
    #[payable]
    pub fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128String, _msg:String, _memo:String){

        //block function-call keys
        assert_one_yocto();

        //transfer
        self.usdnear_transfer(&env::predecessor_account_id(), &receiver_id, amount.0);
    }

    /// Transfer `amount` of tokens from the caller of the contract (`predecessor_id`) to a contract at `receiver_id`.
    /// Requirements:
    /// * receiver_id must be a contract and must respond to `ft_on_transfer(&mut self, sender_id: AccountId, amount: U128String, _msg: String ) -> u128`
    /// * if receiver_id is not a contract or `ft_on_transfer` fails, the transfer is rolled-back
    #[payable]
    pub fn ft_transfer_call(&mut self, receiver_id: AccountId, amount: U128String, msg:String, _memo:String){

        //block function-call keys
        assert_one_yocto();

        //transfer
        self.usdnear_transfer(&env::predecessor_account_id(), &receiver_id, amount.0);

        //call the receiving contract
        ext_ft_receiver::ft_on_transfer(
            env::predecessor_account_id(),
            amount,
            msg,
            //promise params:
            &receiver_id, //contract
            0, //attached native NEAR amount
            100_000_000_000_000, //100TGAS
        )
        .then(ext_self_callback::after_ft_on_transfer_usdnear(
            env::predecessor_account_id(),
            receiver_id,
            amount,
            //promise params:
            &env::current_account_id(),//contract
            0, //attached native NEAR amount
            30_000_000_000_000, //30TGAS
        ));

    }
    /// After Transfer `amount` of symbol tokens to a contract at `receiver_id`.
    /// Check if the contract completed execution of on_multifuntok_transfer
    /// and undo trasnfer if it failed
    pub fn after_ft_on_transfer_usdnear(&mut self, sender_id:AccountId, receiver_id: AccountId, amount: U128String, #[callback] unused_tokens: U128String){

        assert_callback_calling();

        let amt = amount.0;
        if !is_promise_success() {
            //call failed/panicked
            //undo the transfer
            log!("call failed transfer reverted");
            self.usdnear_transfer(&receiver_id, &sender_id, amt);
        }
        else {
            if unused_tokens.0 > 0 {
                //some tokens returned, max to undo is the amount trasnferred
                let undo_amt = std::cmp::min(amt,unused_tokens.0);
                //partially undo the transfer - max to undo is the amount trasnferred
                self.usdnear_transfer(&receiver_id, &sender_id, undo_amt);
                log!("{} unused tokens returned", undo_amt);
            }
        }
    }

}
