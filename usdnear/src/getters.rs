use crate::*;
use near_sdk::{near_bindgen};

#[near_bindgen]
impl UsdNearStableCoin {
    //------------------------------------------
    // GETTERS 
    //------------------------------------------
    /// Returns the account ID of the owner.
    
    pub fn get_operator_account_id(&self) -> AccountId {
        return self.operator_account_id.clone();
    }

    /// The amount of tokens that were deposited to the staking pool.
    /// NOTE: The actual balance can be larger than this known deposit balance due to staking
    /// rewards acquired on the staking pool.
    /// To refresh the amount the owner can call `refresh_staking_pool_balance`.
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
            stbl: acc.stbl.into(),
            outstanding_loans_usdnear: acc.outstanding_loans_usdnear.into(),
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

        self.min_account_balance = params.min_account_balance.0;

        self.usdnear_apr_basis_points = params.usdnear_apr_basis_points;

        self.operator_fee_basis_points = params.operator_fee_basis_points;
        self.treasury_fee_basis_points = params.treasury_fee_basis_points;

    }
    
}
