use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId};
use uint::construct_uint;

pub const META_POOL_STNEAR_CONTRACT: &str = "meta.pool.testnet";

/// useful constants
pub const NO_DEPOSIT: u128 = 0;
pub const NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const ONE_NEAR_CENT: u128 = NEAR/100;
pub const TWO_NEAR: u128 = 2 * NEAR;
pub const FIVE_NEAR: u128 = 5 * NEAR;
pub const TEN_NEAR: u128 = 10 * NEAR;
pub const NEAR_1K: u128 = 1_000 * NEAR;


///To be a liquidator you need to have a borrowing account with at least MIN_STNEAR_BALANCE_FOR_LIQUIDATORS
pub const MIN_STNEAR_BALANCE_FOR_LIQUIDATORS:u128 = 100*NEAR;

pub const PERCENT_BP: u32 = 100; //amount to multyply a percentage to convert to basis points. e.g. 2.5*PERCENT_BP = 250 = 2.5%

pub const DEVELOPERS_ACCOUNT_ID: &str = "developers.near"; 

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

/// Raw type for duration in nanoseconds
pub type Duration = u64;
/// Raw type for timestamp in nanoseconds or Unix Ts in miliseconds
pub type Timestamp = u64;

/// Balance wrapped into a struct for JSON serialization as a string.
pub type U128String = U128;
pub type U64String = U64;

pub type EpochHeight = u64;

/// Hash of Vesting schedule.
pub type Hash = Vec<u8>;

/// NEP-129 get information about this contract
/// returns JSON string according to [NEP-129](https://github.com/nearprotocol/NEPs/pull/129)
/// Rewards fee fraction structure for the staking pool contract.
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
#[allow(non_snake_case)]
pub struct NEP129Response {
    pub dataVersion:u16,
    pub name:String,
    pub version:String,
    pub source:String,
    pub standards:Vec<String>,
    pub webAppUrl:Option<String>,
    pub developersAccountId:String,
    pub auditorAccountId:Option<String>,
}

/// Struct returned from get_account_info
/// div-pool full info
/// Represents account data as as JSON compatible struct
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GetAccountInfoResult {
    pub account_id: AccountId,
    pub usdnear: U128,
    pub stnear: U128,
    pub locked_stnear: U128,
    pub outstanding_loans_usdnear: U128,
    pub collateralization_ratio: u32, //basis points, max 999%
    pub stbl: U128,
}

/// Struct returned from get_contract_state
/// div-pool state info
/// Represents contact state as as JSON compatible struct
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GetContractStateResult {
    pub total_usdnear: U128,
    pub total_collateral_stnear: U128,
    pub current_stnear_price: U128,
    /// total stbl minted
    pub total_stbl: U128, 
    //how many usdnear balances there are
    pub balances_count: U64,
    //how many b_accounts there are
    pub b_accounts_count: U64,
    pub total_collateral_shares: U128,
    pub usdnear_apr_basis_points: u16,
}

/// Struct returned from get_contract_params
/// div-pool parameters info
/// Represents contact parameters as JSON compatible struct
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractParamsJSON {
    pub collateral_basis_points: u32,
    pub min_collateral_basis_points: u32,
    pub borrowing_paused: bool, 
    pub min_account_balance: U128String,
    pub usdnear_apr_basis_points: u16, //2.5% default
    /// operator_fee_basis_points. 100 basis point => 1%. E.g.: owner_fee_basis_points=50 => 0.5% owner's fee
    pub operator_fee_basis_points: u16,
    /// treasury_cut_basis_points. 
    pub treasury_fee_basis_points: u16,
}
