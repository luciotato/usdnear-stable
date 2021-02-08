# USDNEAR Overcollateralized Convertible Stablecoin

## How it works

### Issuance

To issue stablecoins, one needs to deposit some collateral into the stablecoin contract. Collateral is **stNEAR** (A NEP-141 token representing staked NEAR). This collateral will be locked on the stablecoin contract to back the issuance (minting) of USDNEAR stablecoin tokens.

The value of the stablecoin issued is lower than the collateral that is locked to ensure stability in terms of volatility of the collateralized asset, i.e. it is overcollateralized. The overcollateralization parameter is set initially at 200% (i.e. to issue USD 100 worth of stablecoins you need to lock up USD 200 worth of collateral) but can be lowered depeneding on NEAR price volatility.

It’s important to note that a user who requests an issue of USDNEAR 100 is not buying them for USD200 worth of stNEAR. The USD200 worth of stNEAR is a collateral, and the user still owns the collateral and receives USDNEAR 100 stablecoins on top of it. In other words, the issuance of a stablecoin is the issuance of a collateralized loan in the form of a stablecoin. And the one who requests an issuance of the stablecoin is a borrower who locks up their collateral to secure the loan. To get the collateral back, they will need to return the same amount of stablecoins they borrowed.

The total supply of stablecoins increases as a new loan is issued, and decreases as a loan is repaid and the repaid stablecoins burned. So, the stablecoin is fully backed by a 200% collateral.

### Interest

The interest on the issued USDNEAR will be computed as to be 4% Effective Annual Interest Rate, and collected on each epoch in the same process of computing staking rewards.
Because staked NEAR receive staking rewards each epoch (every 12hs) the collateral will tend to naturally increase in value. The APY for staked near is around 10% now (Feb-2021), so initially the rewards will be more than enough to cover the interest on the loans.

### Collateral Price Oracle

Collateralization levels are determined by using price data reported by an oracle external to the smart contract. The price data should ideally not be based on the instantaneous price but rather a moving average (MA) over some sufficiently long period. Every hour, the contract operator computes the average NEAR price in USD from one or more exchange markets and send the information to the smart contract. That price determines the ratio of collateralization and the exchange rate for the conversion window.

### Conversion Window

Any USDNEAR token owner can convert their token into stNEAR. The amount of stNEAR the user receives is computed from the market price of NEAR so the conversion is always "1 USDNEAR" = "1 USD worth of stNEAR". The conversion window transfer the amount of stNEAR from the collateral pool to the user, and receives and burns USDNEAR. Since USDNEAR is burned, the total Overcollateralization ratio remains the same.  This conversion mechanism does not change any of the existing loans. The users with outstanding loans still need to repay their loans in order to free their collateral.

The existence of a conversion fixed-rate window where "1 USDNEAR" = "1 USD worth of stNEAR", guarantees the stability of USDNEAR. The overcollateralization guarantees conversion availability, protecting USDNEAR holders form the collateral asset (stNEAR) price volatility.

The main users exchanging USDNEAR on the conversion window are those users that received USDNEAR token as payment or in exchange for other assets and want to convert USDNEAR to stNEAR. Other token owners could choose to expend their USDNEAR, keeping the coins on circulation. The users with oustanding loans use the USDNEAR they receive to reduce or repay their loans.

### Minimum collateralization and Liquidation

The large 200% collateral is needed to protect against the inherent volatility of the native NEAR token. The value of collateral must always be greater than the value of stablecoins issued against it. To ensure that the collateral is always sufficient, the borrower has an obligation to always keep it above a minimum collateralization ratio, let’s say it is 150% (can be changed in the contract parameters). The borrower can do so by sending additional collateral to the loan, or repaying the loan before its collateralization gets too low.

If the borrower fails to refill the collateral on time and the actual collateralization ratio drops below the minimum, the loan is "open for liquidation". Any other user with an account in the contract can "buy" part of the loan by sending USDNEAR, and receives in return 110% worth of stNEAR from the collateral (10% profit)

The USDNEAR is used to partially repay the loan in order to raise the collateralization ratio up to 175%. The max amount of USDNEAR that the liquidator can "repay" is computed as to raise the collateralization ratio up to 175% (the average between the initial and minimun collateralization ratio)

Liquidators must have their own mechanism to identify "open for liquidation" loans. The contract will provide an API to inform (in batches) the state of all the oustanding loans. The first valid transaction buying the loan gets the 10% profit liquidation fee.

## Use Cases

Alice can have an account where she can deposit collateral stNEAR. 
Some minimun NEAR deposit may be required to open the account (to back-up account storage) if that storage is not provided by the operator.

Once Alice deposits the collateral, a "credit limit" is computed according to the current NEAR price and the collateralization ratio. e.g. if Alice deposited stNEAR 100, and the NEAR price is USD 2.5, then the collateral value is USD 250. If the collateralization ratio is 200% it means Alice will have a "line of credit" to borrow USDNEAR 125 backed by her USD 250 valued collateral.

Alice account now reads:
- Tokens
-- USDNEAR: 0
-- stNEAR: 100 Available (USD 250) + 0 Locked (USD 0)
- Line of Credit
-- Limit: USDNEAR 125 [Take Loan]
- Outstanding Loans:
-- owed: USDNEAR 0
-- Locked Collateral: stNEAR 0


If Alice chooses to borrow USDNEAR 100, USDNEAR 100 are minted into Alice's account, and USD 200 value of collateral is locked and trasnferred to the collateral pool. The global amount of minted USDNEAR will be increased by 100 and the value of the collateral pool will be incresed by stNEAR valued USD 200, keeping the global overcollateralization constant.

Alice account now reads:
- Tokens  
-- USDNEAR: 100  
-- stNEAR: 20 Available (USD 50) + 80 Locked (USD 200)  
- Line of Credit  
-- Limit: USDNEAR 50  
- Outstanding Loans:  
-- owed: USDNEAR 100   [Repay]  
-- Locked stNEAR (Collateral): 80 (USD 200) [Add Collateral]  
-- Collateralization Ratio: 200%  

Let's assume Alice trasnfers USDNEAR 100 to Bob

Bob account now reads:
- Tokens  
-- USDNEAR: 100  

and Alice account reads:
- Tokens  
-- USDNEAR: 0  
-- stNEAR: 20 Available (USD 50) + 80 Locked (USD 200)  
- Line of Credit  
-- Limit: USDNEAR 50  
- Outstanding Loans:  
-- owed: USDNEAR 100   [Repay]  
-- Locked stNEAR (Collateral): 80 (USD 200) [Add Collateral]  
-- Collateralization Ratio: 200%  
### Liquidation Event

Let's assume the NEAR price is now USD 1.50. With this new price Alice's account reads:

- Tokens  
-- USDNEAR: 0  
-- stNEAR: 20 Available (USD 30) + 80 Locked (USD 120)  
- Line of Credit  
-- Limit: USDNEAR 50  
- Outstanding Loans:  
-- owed: USDNEAR 100   [Repay]  
-- Locked stNEAR (Collateral): 80 (USD 120)   
-- Collateralization Ratio: 120% **OPEN FOR LIQUIDATION** [Add Collateral]  

At this point Alice's debt is open for liquidation because it's collateral is less than 150%


-------------------

# Technical Notes

## Operator Functions

There are several functions for the operator to periodically call:

### 1. set_reference_price()

Every hour, the contract operator computes the average NEAR price in USD from one or more exchange markets and send the information to the smart contract. That price determines the ratio of collateralization and the exchange rate for the conversion window.

###  2. compute_rewards()

In the middle of each epoch, the contract operator calls this function to compute staking rewards from all the collateral. The rewards are added to the collateral pool, increasing the stNEAR amount and then collateralization for all users.

###  3. collect_interest()

In the middle of each epoch, after calling compute_rewards(), the contract operator calls this function to compute interest on collateral. Interest is sent to the contract treasury.

