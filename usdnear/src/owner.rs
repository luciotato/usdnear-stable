//
// OWNER'S METHODS & GET info methods
//

use crate::*;
use near_sdk::{near_bindgen, Promise, PublicKey};

#[near_bindgen]
impl UsdNearStableCoin {

    /// Owner's method.
    /// Adds full access key with the given public key to the account once the contract is empty (has no b_accounts)
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

    /// Returns the account ID of the owner.
    pub fn get_operator_account_id(&self) -> AccountId {
        return self.operator_account_id.clone();
    }

    /// The amount of usdnear tokens in circulation
    pub fn get_total_usdnear(&self) -> U128String {
        return self.total_usdnear.into();
    }

    /// full account info
    /// Returns JSON representation of the account for the given account ID.
    pub fn get_account_info(&self, account_id: AccountId) -> GetAccountInfoResult {
        let usdnear = self.usdnear_balances.get(&account_id).unwrap_or_default(); 
        let acc = self.internal_get_account(&account_id);
        let stnear = self.amount_from_collateral_shares(acc.collateral_shares);
        return GetAccountInfoResult {
            account_id,
            usdnear: usdnear.into(),
            stnear: stnear.into(),
            stnear_price_usd: self.current_stnear_price.into(),
            stbl: acc.stbl.into(),
            usdnear_credit_limit: acc.get_current_credit_limit(self).into(),
            outstanding_loans_usdnear: acc.outstanding_loans_usdnear.into(),
            locked_stnear: acc.locked_collateral_stnear(&self).into(),
            collateralization_ratio: acc.get_current_collateralization_ratio(&self),
        };
    }

    /// Returns the number of borrowing accounts 
    pub fn get_number_of_accounts(&self) -> u64 {
        return self.b_accounts.len();
    }

    /// Returns a partial list of borrowing accounts 
    pub fn get_accounts(&self, from_index: u64, limit: u32) -> Vec<GetAccountInfoResult> {
        assert!(limit<10000);
        let keys = self.b_accounts.keys_as_vector();
        return (from_index..std::cmp::min(from_index + limit as u64, keys.len()))
            .map(|index| self.get_account_info(keys.get(index).unwrap()))
            .collect();
    }

    /// NEP-129 get information about this contract
    /// returns JSON string according to [NEP-129](https://github.com/nearprotocol/NEPs/pull/129)
    pub fn get_contract_info(&self) -> NEP129Response {
        return NEP129Response {
            dataVersion:1,
            name: CONTRACT_NAME.into(),
            version:CONTRACT_VERSION.into(),
            source:"https://github.com/Narwallets/usdnear-stable".into(), 
            standards:vec!("NEP-129".into(),"NEP-138".into()),  
            webAppUrl:self.web_app_url.clone(),
            developersAccountId:DEVELOPERS_ACCOUNT_ID.into(),
            auditorAccountId: self.auditor_account_id.clone()
        }
    }

    /// sets configurable contract info [NEP-129](https://github.com/nearprotocol/NEPs/pull/129)
    // Note: params are not Option<String> so the user can not inadvertely set null to data by not including the argument
    pub fn set_contract_info(&mut self, web_app_url:String, auditor_account_id:String) {
        self.assert_owner_calling();
        self.web_app_url = if web_app_url.len()>0 { Some(web_app_url) } else { None };
        self.auditor_account_id = if auditor_account_id.len()>0 { Some(auditor_account_id) } else { None };
    }

    /// get contract totals 
    /// Returns JSON representation of the contract state
    pub fn get_contract_state(&self) -> GetContractStateResult {

        return GetContractStateResult {
            total_usdnear: self.total_usdnear.into(),
            total_collateral_stnear: self.total_collateral_stnear.into(),
            current_stnear_price: self.current_stnear_price.into(),
            valued_collateral: self.stnear_to_usd(self.total_collateral_stnear).into(),
            total_stbl: self.total_stbl.into(),
            balances_count: self.usdnear_balances.len().into(),
            b_accounts_count: self.b_accounts.len().into(),
            total_collateral_shares: self.total_collateral_shares.into(),
            usdnear_apr_basis_points: self.usdnear_apr_basis_points,
        };
    }

    /// Returns JSON representation of contract parameters
    pub fn get_contract_params(&self) -> ContractParamsJSON {
        return ContractParamsJSON {
            collateral_basis_points: self.collateral_basis_points,
            min_collateral_basis_points: self.min_collateral_basis_points,
            borrowing_paused: self.borrowing_paused,
            min_account_balance: self.min_account_balance.into(),
            usdnear_apr_basis_points: self.usdnear_apr_basis_points,
            epochs_per_year: self.epochs_per_year,
            operator_fee_basis_points: self.operator_fee_basis_points,
            treasury_fee_basis_points: self.treasury_fee_basis_points,
            };
    }

    /// Sets contract parameters 
    pub fn set_contract_params(&mut self, params:ContractParamsJSON) {

        self.assert_owner_calling();

        assert!(params.collateral_basis_points>120*PERCENT_BP);
        self.collateral_basis_points = params.collateral_basis_points;

        assert!(params.min_collateral_basis_points>110*PERCENT_BP);
        self.min_collateral_basis_points = params.min_collateral_basis_points;

        // collateral_basis_points should be > 100%collat+liquidation_fee 
        assert!(self.collateral_basis_points > 10000+self.liquidaton_fee_basis_points as u32);

        self.borrowing_paused = params.borrowing_paused;

        self.min_account_balance = params.min_account_balance.0;

        self.usdnear_apr_basis_points = params.usdnear_apr_basis_points;
        self.epochs_per_year = params.epochs_per_year;

        assert!(params.operator_fee_basis_points+params.treasury_fee_basis_points==10000,"fee split must add 100%");
        self.operator_fee_basis_points = params.operator_fee_basis_points;
        self.treasury_fee_basis_points = params.treasury_fee_basis_points;
    }


    /// Sets contract parameters 
    pub fn set_stnear_price_usd(&mut self, stnear_price_usd:U128String) {
        self.assert_owner_calling();
        //allow 25% variation max
        assert!(stnear_price_usd.0 > self.current_stnear_price* 75/100 && stnear_price_usd.0 < self.current_stnear_price * 125/100 );
        self.current_stnear_price = stnear_price_usd.0;
    }
    
    pub fn clear_busy_flag(&mut self) {
        self.assert_owner_calling();
        self.busy= false;
    }

    /// compute rewards and interest
    //------------------------------------------------
    //-- COMPUTE STAKING REWARDS and collect interest
    //------------------------------------------------
    // Operator method, but open to anyone. Should be called once per epoch
    /// Retrieves actual balance from the stNEAR contract 
    /// this fn queries the Meta-staking-pool contract (makes a cross-contract call)
    pub fn compute_rewards_and_interest(&mut self) {

        //Note: In order to make this contract independent from the operator
        //this fn is open to be called by anyone
        //self.assert_owner_calling();

        self.assert_not_busy();

        let epoch_height = env::epoch_height();
        if self.last_rewards_epoch_height == epoch_height {
            panic!("already run in this epoch");
        }

        self.busy=true;

        //query our current balance (includes staking rewards)
        ext_meta_pool::get_account_total_balance(
            env::current_account_id(),
            //promise params
            &String::from(META_POOL_STNEAR_CONTRACT),
            NO_DEPOSIT,
            gas::GET_ACCOUNT_TOTAL_BALANCE,
        )
        .then(ext_self_callback::after_get_meta_contract_stnear_total_balance(
            //promise params
            &env::current_account_id(),
            NO_DEPOSIT,
            gas::AFTER_GET_ACCOUNT_TOTAL_BALANCE,
        ));
    }
    /// prev fn continues here - must not panic
    //-----------------------------------------
    pub fn after_get_meta_contract_stnear_total_balance(
        &mut self,
        #[callback] total_balance: U128String,
    ) {
        //we enter here after asking the meta-staking-pool how much do we have staked (plus rewards)
        //total_balance: U128String contains the answer from the meta-staking-pool

        assert_callback_calling();
        self.busy=false;

        let new_staked_amount= total_balance.0;

        let rewards: u128;
        if new_staked_amount < self.total_collateral_stnear {
            log!(
                "INCONSISTENCY: meta-contract says total stNEAR {} < self.total_collateral_stnear {}",
                new_staked_amount , self.total_collateral_stnear
            );
            rewards = 0;
        } else {
            //compute rewards, as new balance minus old balance
            rewards = new_staked_amount - self.total_collateral_stnear;
        }

        log!(
            "meta-contract says: old stNEAR:{} new:{} rewards:{}",
            self.total_collateral_stnear, new_staked_amount, rewards
        );

        if rewards > 0 {
            // compute interest on loans as a % of rewards
            // initially loan interest is 2.5% APR. 
            // NEAR staking rewards are about 10% APY (feb-2021), so a 25% of rewards approximates 2.5% APR
            // The 25% has to be computed ONLY on 100% collateral, so is 2.5% APR over USDNEAR owed

            let interest_year_usdnear = apply_pct(self.usdnear_apr_basis_points, self.total_usdnear);
            let interest_epoch_usdnear = interest_year_usdnear / self.epochs_per_year as u128; //default 365*2, 2 epochs per day
            let interest_epoch_stnear = self.usdnear_to_stnear(interest_epoch_usdnear);

            //add interest stNEAR to treasury
            &self.add_amount_and_shares_preserve_share_price(self.treasury_account_id.clone(),interest_epoch_stnear);
            log!("treasury got {} as epoch interest",interest_epoch_stnear);
            
            // rest of staking rewards go into total_collateral increasing share value -> stNEAR amounts for everyone
            self.total_collateral_stnear += rewards - interest_epoch_stnear; 

            self.last_rewards_epoch_height = env::epoch_height();

        }
    }
    

}
