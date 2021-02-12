# USDNEAR Overcollateralized Convertible Stablecoin

The first Stablecoin for the NEAR Ecosystem, backed by staking, with self-paid low interest, and convertible.

## Advantages

*Trust*: We chose a simple design to make the system transparent, easy to understand, and highly reliable. The contract has a  Conversion Window where USDNEAR owners can exchange their tokens for stNEAR from the collateral pool. The rate is fixed, so "1 USDNEAR" always equals "1 USD woth of stNEAR" at the current market price. The over-collaterization rate is 200%, so there is ample funds to feed the conversion window.

*Ease of use*: NEAR accounts are "named accounts" where users choose their account name, e.g.: 'alice.near'. Having readable account names, very low fees, and a stablecoin are drivers for mass adoption outside of the crypto enthusiast circle. A stablecoin with fees less than a cent and readable account names becomes useful and usable.

*Fees*: Stablecoins on Ethereum are hampered by astronomical fees. The fee to send DAI 100 could reach USD 15 or more. Ethereum fees make low-amount operation impossible. On the other hand, the fees on the NEAR blockchain are tenths of a cent, so minting USDNEAR and sending the tokens costs only a fraction of a cent.

*Auto-paid Interest*: Staked Near receives staking rewards every 12 hours (each NEAR Epoch), so the interest on the loan is paid from the staking rewards, the collateral naturally increases and never decreases. The whole process is transparent and automatic on the NEAR Blockchain. Users can keep reap staking rewards and at the same time mint a stablecoin to use without the volatility of the market.


## How it works

### Issuance

To issue stablecoins, one needs to deposit some collateral into the stablecoin contract. Collateral is **stNEAR** (A NEP-141 token representing staked NEAR). This collateral will be locked on the stablecoin contract to back the issuance (minting) of USDNEAR stablecoin tokens.

The value of the stablecoin issued is lower than the collateral that is locked to ensure stability in terms of volatility of the collateralized asset, i.e. it is overcollateralized. The overcollateralization parameter is set initially at 200% (i.e. to issue USD 100 worth of stablecoins you need to lock up USD 200 worth of collateral) but can be lowered depeneding on NEAR price volatility.

It’s important to note that a user who requests an issue of USDNEAR 100 is not buying them for USD200 worth of stNEAR. The USD200 worth of stNEAR is a collateral, and the user still owns the collateral and receives USDNEAR 100 stablecoins on top of it. In other words, the issuance of a stablecoin is the issuance of a collateralized loan in the form of a stablecoin. And the one who requests an issuance of the stablecoin is a borrower who locks up their collateral to secure the loan. To get the collateral back, they will need to return the same amount of stablecoins they borrowed.

The total supply of stablecoins increases as a new loan is issued, and decreases as a loan is repaid and the repaid stablecoins burned. So, the stablecoin is fully backed by a 200% collateral.

### Interest

The interest on the issued USDNEAR will be collected on each epoch in the same process of computing staking rewards.
Considering that staked NEAR receive staking rewards each epoch (every 12hs) the collateral naturally increase in value every 12hs. 
The APY for staked near is around 10% now (Feb-2021), so initially the rewards will be more than enough to cover the initial 2.5% APR on USDNEAR loans.
The 2.5% APR is taken proportionally from rewards on each epoch, so if the amount of rewards goes down the APR will go down too, so 2.5% APR is configurable target and not an exact figure. Effective APR can be lower.

### Collateral Price Oracle

Collateralization levels are determined by using price data reported by an oracle external to the smart contract. The price data should ideally not be based on the instantaneous price but rather a moving average (MA) over some sufficiently long period. Every hour, the contract operator computes the average NEAR price in USD from one or more exchange markets and send the information to the smart contract. That price determines the ratio of collateralization and the exchange rate for the conversion window.

### Conversion Window

Any USDNEAR token owner can convert their token into stNEAR. The amount of stNEAR the user receives is computed from the market price of NEAR so the conversion is always "1 USDNEAR" = "1 USD worth of stNEAR". The conversion window transfers the amount of stNEAR from the collateral pool to the user, and receives and burns USDNEAR. Since USDNEAR is burned, the total Overcollateralization ratio remains the same. 

The existence of a conversion fixed-rate window where "1 USDNEAR" = "1 USD worth of stNEAR", guarantees the stability of USDNEAR. The overcollateralization guarantees conversion availability, protecting USDNEAR holders form the collateral asset (stNEAR) price volatility.

The main users exchanging USDNEAR on the conversion window are those users that received USDNEAR token as payment or in exchange for other assets and want to convert USDNEAR to stNEAR. Other token owners could choose to expend their USDNEAR, keeping the coins on circulation. The users with oustanding loans use the USDNEAR they receive to reduce or repay their loans.

The conversion is paid from the user's locked collateral. After a conversion all users with outstanding loans will owe a little less (USDNEAR was burned) but they will also have that USD amount deducted from their locked collateral value. In the extreme case that all the USDNEAR in circulation is converted and burned, all loans will be reduced to zero (no more USDNEAR in circulation) and all locked collaterals (users with outstanding loans) would have been reduced by the same USD amount.  

### Minimum collateralization and Liquidation

The large 200% collateral is needed to protect against the inherent volatility of the native NEAR token. The value of collateral must always be greater than the value of stablecoins issued against it. To ensure that the collateral is always sufficient, the borrower has an obligation to always keep it above a minimum collateralization ratio, let’s say it is 150% (can be changed in the contract parameters). The borrower can do so by sending additional collateral to the loan, or repaying the loan before its collateralization gets too low.

If the borrower fails to refill the collateral on time and the actual collateralization ratio drops below the minimum, the loan is "open for liquidation". Any other user with an account in the contract can "buy" part of the loan by sending USDNEAR, and receives in return 110% worth of stNEAR from the collateral (10% profit)

The USDNEAR is used to partially repay the loan in order to raise the collateralization ratio up to 200%. The max amount of USDNEAR that the liquidator can "repay" is computed as to restore the collateralization ratio to 200%

Liquidators must have their own mechanism to identify "open for liquidation" loans. The contract will provide an API to inform (in batches) the state of all the oustanding loans. The first valid transaction buying the loan gets the 10% profit liquidation fee.

## Use Cases

Alice creates an account when she deposits collateral stNEAR into the contract. 
(Normally some NEAR is required to back-up account storage, but that NEAR is provided by the operator or the treasury. NEAR per storage price is being lowered to 1N per 10Kib).

Once Alice deposits the collateral, a "credit limit" is computed according to the current stNEAR price and the collateralization ratio. e.g. if Alice deposited stNEAR 100, and the stNEAR price is USD 2.5, then the collateral value is USD 250. If the collateralization ratio is 200% it means Alice will have a "line of credit" to borrow USDNEAR 125 backed by her USD 250 valued collateral.

Alice account now reads:
- Tokens
  - USDNEAR: 0 
- Line of Credit
  - USDNEAR: 125 [Take Loan]
- Collateral
  - Total: stNEAR: 100 (USD 250) [Add Collateral]
  - Locked: stNEAR 0
- Outstanding Loans:
  - owed: USDNEAR 0


If Alice chooses to borrow USDNEAR 100, USDNEAR 100 are minted into Alice's account, and USD 200 value of collateral is locked. The global amount of minted USDNEAR will be increased by 100 and the value of the locked collateral will be incresed by 80 stNEAR valued USD 200, keeping the global overcollateralization constant.

Alice account now reads:
- Tokens  
  - USDNEAR: 100  [Transfer]
- Line of Credit  
  - USDNEAR: 50  [Take Loan]
- Collateral
  - Total: stNEAR: 100 (USD 250) [Add Collateral]
  - Locked: stNEAR 80 (USD 200) 
  - Free: stNEAR 20 (USD 50) 
- Outstanding Loans:  
  - owed: USDNEAR 100   [Repay]  
  - Collateralization Ratio: 200%  

Let's assume Alice trasnfers USDNEAR 100 to Bob

Bob account now reads:
- Tokens  
  - USDNEAR: 100 [Transfer]

and Alice account reads:
- Tokens  
  - USDNEAR: 0  [Transfer]
- Line of Credit  
  - Limit: USDNEAR 50  
- Collateral
  - Total: stNEAR: 100 (USD 250) [Add Collateral]
  - Locked: stNEAR 80 (USD 200) 
  - Free: stNEAR 20 (USD 50) 
- Outstanding Loans:  
  - owed: USDNEAR 100   [Repay]  
  - Collateralization Ratio: 200%  

### Liquidation Event

Let's assume the NEAR price is now USD 1.4240. With this new price Alice's account reads:

- Tokens  
  - USDNEAR: 0  
- Line of Credit  
  - Limit: USDNEAR 0
- Collateral
  - Total: stNEAR: 100 (USD 142.40) [Add Collateral]
  - Locked: stNEAR 100 (USD 142.40) 
  - Free: stNEAR 0 (USD 0) 
- Outstanding Loans:  
  - owed: USDNEAR 100   [Repay]  
  - Collateralization Ratio: 142.4% **OPEN FOR LIQUIDATION** 

Even using all Alice's collateal, the Collateralization Ratio is below 150%.

At this point Alice's debt is open for liquidation because it's collateral is less than 150%. 

A liquidator will be able to repay USDNEAR 64 from Alice's loan and will receive USD 70.4 worth of stNEAR from Alice's locked collateral (10% profit).

After repaying USDNEAR 64, Alice's new owed amount is USDNEAR 36, and her remainig collateral is valued USD 72 so her new Collateralization Ratio is 72/36 => 200%

The amount a liquidator can repay is computed as: 
```
open_to_liquidate_USDNEAR = (collateral_value - 200% * owed) / (110% - 200%) 
```
so after liquidation the Collateralization Ratio is back to 200% considering a 10% liquidation fee.

After Liquidation, Alice's account reads:

- Tokens  
  - USDNEAR: 0  
- Line of Credit  
  - Limit: USDNEAR 0
- Collateral
  - Total: stNEAR: 50.5618 (USD 72) [Add Collateral]
  - Locked: stNEAR 50.5618 (USD 72)
  - Free: stNEAR 0 (USD 0) 
- Outstanding Loans:  
  - owed: USDNEAR 36 [Repay]  
  - Collateralization Ratio: 200%


-------------------

# Technical Notes

## Operator Functions

There are several functions for the operator to periodically call:

### 1. set_stnear_price_usd()

Every hour, the contract operator computes the average NEAR/stNEAR price in USD from one or more exchange markets and send the information to the smart contract. That price determines the ratio of collateralization and the exchange rate for the conversion window.

###  2. compute_rewards_and_interest()

Each epoch, the contract operator calls this function to compute staking rewards from all the collateral. The rewards are added to the collateral pool, increasing the stNEAR amount and then collateralization for all users.

###  3. collect_interest()

In the middle of each epoch, after calling compute_rewards(), the contract operator calls this function to compute interest on collateral. Interest is sent to the contract treasury.

