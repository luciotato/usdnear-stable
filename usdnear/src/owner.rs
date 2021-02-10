    /// OWNER'S METHOD & get info methods
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
            locked_stnear: acc.locked_collateral_stnear(&self).into(),
            stbl: acc.stbl.into(),
            outstanding_loans_usdnear: acc.outstanding_loans_usdnear.into(),
            collateralization_ratio: acc.get_current_collateralization_ratio(&self),
        };
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
            total_stbl: self.total_stbl.into(),
            balances_count: self.usdnear_balances.len().into(),
            b_accounts_count: self.b_accounts.len().into(),
            current_stnear_price: self.current_stnear_price.into(),
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

        assert!(params.operator_fee_basis_points+params.treasury_fee_basis_points==10000,"fee split must add 100%");
        self.operator_fee_basis_points = params.operator_fee_basis_points;
        self.treasury_fee_basis_points = params.treasury_fee_basis_points;
    }
    
}
