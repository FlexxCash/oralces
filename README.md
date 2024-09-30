# Solana BNPL Smart Contract System

This project implements a Buy Now, Pay Later (BNPL) smart contract system on the Solana blockchain, focusing on price oracle functionality.

## Project Structure

```
flexxcash_bnpl/
│
├── programs/
│   └── oracles/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── price_oracle.rs
│
├── tests/
│   └── price_oracle.ts
│
├── Anchor.toml
├── README.md
└── Cargo.toml
```

## Dependencies

Main dependencies and their versions (defined in Cargo.toml):

- anchor-lang = { version = "0.28.0", features = ["init-if-needed"] }
- anchor-spl = "0.28.0"
- switchboard-v2 = "0.4.0"
- serde = { version = "1.0", features = ["derive"] }
- serde_json = "1.0"

## File Descriptions

### programs/oracles/src/lib.rs

This is the main program entry point, defining instructions for interacting with the price oracle.

#### Functions

1. `initialize(ctx: Context<Initialize>) -> Result<()>`
   - Purpose: Initializes the price oracle account.
   - Parameters: `ctx` - Context containing accounts needed for initialization.
   - Returns: `Ok(())` on success.

2. `update_price(ctx: Context<UpdatePrice>, asset_type: AssetType) -> Result<()>`
   - Purpose: Updates the price for a specified asset type.
   - Parameters:
     - `ctx` - Context containing accounts needed for price update.
     - `asset_type` - The type of asset to update the price for.
   - Returns: `Ok(())` on success.

3. `update_apy(ctx: Context<UpdateApy>, asset_type: AssetType) -> Result<()>`
   - Purpose: Updates the APY for a specified asset type.
   - Parameters:
     - `ctx` - Context containing accounts needed for APY update.
     - `asset_type` - The type of asset to update the APY for.
   - Returns: `Ok(())` on success.

4. `get_current_price(ctx: Context<GetCurrentPrice>, asset_type: AssetType) -> Result<()>`
   - Purpose: Retrieves the current price for a specified asset type.
   - Parameters:
     - `ctx` - Context containing accounts needed to get the price.
     - `asset_type` - The type of asset to get the price for.
   - Returns: `Ok(())` on success, logs the price.

5. `get_current_apy(ctx: Context<GetCurrentApy>, asset_type: AssetType) -> Result<()>`
   - Purpose: Retrieves the current APY for a specified asset type.
   - Parameters:
     - `ctx` - Context containing accounts needed to get the APY.
     - `asset_type` - The type of asset to get the APY for.
   - Returns: `Ok(())` on success, logs the APY.

6. `get_sol_price(ctx: Context<GetSolPrice>) -> Result<()>`
   - Purpose: Retrieves the current price of SOL.
   - Parameters: `ctx` - Context containing accounts needed to get the SOL price.
   - Returns: `Ok(())` on success, logs the SOL price.

7. `get_usdc_price(_ctx: Context<GetUsdcPrice>) -> Result<()>`
   - Purpose: Retrieves the current price of USDC (fixed at $1.00).
   - Parameters: `_ctx` - Unused context parameter.
   - Returns: `Ok(())` on success, logs the USDC price.

8. `check_emergency_stop(ctx: Context<CheckEmergencyStop>) -> Result<()>`
   - Purpose: Checks if the emergency stop is activated.
   - Parameters: `ctx` - Context containing the price oracle account.
   - Returns: `Ok(())` on success, logs the emergency stop status.

9. `set_emergency_stop(ctx: Context<SetEmergencyStop>, stop: bool) -> Result<()>`
   - Purpose: Sets the emergency stop status.
   - Parameters:
     - `ctx` - Context containing the price oracle account.
     - `stop` - Boolean value to set the emergency stop status.
   - Returns: `Ok(())` on success, logs the new emergency stop status.

#### Structs

1. `Initialize<'info>`
   - Purpose: Defines accounts needed for initialization.
   - Fields:
     - `price_oracle: Account<'info, PriceOracle>` - The price oracle account to initialize.
     - `user: Signer<'info>` - The user initializing the account.
     - `system_program: Program<'info, System>` - The system program.

2. `UpdatePrice<'info>`
   - Purpose: Defines accounts needed for updating price.
   - Fields:
     - `price_oracle: Account<'info, PriceOracle>` - The price oracle account.
     - `oracle_feed: AccountLoader<'info, AggregatorAccountData>` - The Switchboard oracle feed.

3. `UpdateApy<'info>`
   - Purpose: Defines accounts needed for updating APY.
   - Fields:
     - `price_oracle: Account<'info, PriceOracle>` - The price oracle account.
     - `oracle_feed: AccountLoader<'info, AggregatorAccountData>` - The Switchboard oracle feed.

4. `GetCurrentPrice<'info>`
   - Purpose: Defines accounts needed for getting current price.
   - Fields:
     - `price_oracle: Account<'info, PriceOracle>` - The price oracle account.

5. `GetCurrentApy<'info>`
   - Purpose: Defines accounts needed for getting current APY.
   - Fields:
     - `price_oracle: Account<'info, PriceOracle>` - The price oracle account.

6. `GetSolPrice<'info>`
   - Purpose: Defines accounts needed for getting SOL price.
   - Fields:
     - `price_oracle: Account<'info, PriceOracle>` - The price oracle account.

7. `GetUsdcPrice`
   - Purpose: Defines accounts needed for getting USDC price (empty struct as USDC price is fixed).

8. `CheckEmergencyStop<'info>`
   - Purpose: Defines accounts needed for checking emergency stop status.
   - Fields:
     - `price_oracle: Account<'info, PriceOracle>` - The price oracle account.

9. `SetEmergencyStop<'info>`
   - Purpose: Defines accounts needed for setting emergency stop status.
   - Fields:
     - `price_oracle: Account<'info, PriceOracle>` - The price oracle account.
     - `authority: Signer<'info>` - The authority allowed to set emergency stop.

#### Error Codes

`ErrorCode` enum defines possible error types:

- `PriceChangeExceedsLimit` - Price change exceeds the allowed limit.
- `OracleError` - An error occurred in the oracle.
- `InvalidAssetType` - The asset type is invalid.
- `PriceNotAvailable` - The price is not available.
- `ApyNotAvailable` - The APY is not available.
- `EmergencyStop` - The emergency stop is activated.

### programs/oracles/src/price_oracle.rs

This file implements the core logic of the price oracle.

#### Enums

`AssetType` enum defines supported asset types:
- JupSOL, MSOL, BSOL, HSOL, JitoSOL, VSOL, SOL

#### Structs

1. `PriceData`
   - Purpose: Stores price-related data.
   - Fields:
     - `price: f64` - Current price.
     - `last_price: f64` - Previous price.
     - `last_update_time: i64` - Timestamp of the last update.

     - `apy: f64` - Current APY.

2. `PriceOracleHeader`
   - Purpose: Stores global oracle data.
   - Fields:
     - `asset_count: u8` - Number of assets tracked.
     - `last_global_update: i64` - Timestamp of the last global update.
     - `emergency_stop: bool` - Emergency stop flag.
     - `authority: Pubkey` - Authority public key.
     - `bump: u8` - PDA bump.

3. `PriceOracleData`
   - Purpose: Stores price data for all assets.
   - Fields:
     - `price_data: Vec<PriceData>` - Vector of price data for each asset.
     - `asset_types: Vec<AssetTypeWrapper>` - Vector of asset types.
     - `bump: u8` - PDA bump.

#### Functions

1. `AssetType::get_feed_address(&self) -> Result<Pubkey>`
   - Purpose: Gets the Switchboard feed address for a given asset type.
   - Parameters: `self` - The AssetType instance.
   - Returns: `Result<Pubkey>` - The Pubkey of the feed address or an error.

2. `PriceOracle::initialize(&mut self) -> Result<()>`
   - Purpose: Initializes the PriceOracle struct.
   - Parameters: `self` - Mutable reference to the PriceOracle instance.
   - Returns: `Result<()>` - Ok(()) on success.

3. `PriceOracle::update_price(&mut self, feed: &AccountLoader<AggregatorAccountData>, asset_type: AssetType, clock: &Clock) -> Result<()>`
   - Purpose: Updates the price for a given asset type.
   - Parameters:
     - `self` - Mutable reference to the PriceOracle instance.
     - `feed` - Reference to the Switchboard feed account.
     - `asset_type` - The asset type to update.
     - `clock` - Reference to the system clock.
   - Returns: `Result<()>` - Ok(()) on success.

4. `PriceOracle::update_apy(&mut self, feed: &AccountLoader<AggregatorAccountData>, asset_type: AssetType, clock: &Clock) -> Result<()>`
   - Purpose: Updates the APY for a given asset type.
   - Parameters:
     - `self` - Mutable reference to the PriceOracle instance.
     - `feed` - Reference to the Switchboard feed account.
     - `asset_type` - The asset type to update.
     - `clock` - Reference to the system clock.
   - Returns: `Result<()>` - Ok(()) on success.

5. `PriceOracle::get_current_price(&self, asset_type: AssetType) -> Result<f64>`
   - Purpose: Gets the current price for a given asset type.
   - Parameters:
     - `self` - Reference to the PriceOracle instance.
     - `asset_type` - The asset type to get the price for.
   - Returns: `Result<f64>` - The current price or an error.

6. `PriceOracle::get_last_price(&self, asset_type: AssetType) -> Result<f64>`
   - Purpose: Gets the last price for a given asset type.
   - Parameters:
     - `self` - Reference to the PriceOracle instance.
     - `asset_type` - The asset type to get the last price for.
   - Returns: `Result<f64>` - The last price or an error.

7. `PriceOracle::get_current_apy(&self, asset_type: AssetType) -> Result<f64>`
   - Purpose: Gets the current APY for a given asset type.
   - Parameters:
     - `self` - Reference to the PriceOracle instance.
     - `asset_type` - The asset type to get the APY for.
   - Returns: `Result<f64>` - The current APY or an error.

8. `PriceOracle::last_update_time(&self, asset_type: AssetType) -> Result<i64>`
   - Purpose: Gets the last update time for a given asset type.
   - Parameters:
     - `self` - Reference to the PriceOracle instance.
     - `asset_type` - The asset type to get the last update time for.
   - Returns: `Result<i64>` - The last update timestamp or an error.

9. `PriceOracle::is_emergency_stopped(&self) -> bool`
   - Purpose: Checks if the emergency stop is activated.
   - Parameters: `self` - Reference to the PriceOracle instance.
   - Returns: `bool` - True if emergency stop is activated, false otherwise.

10. `PriceOracle::set_emergency_stop(&mut self, stop: bool)`
    - Purpose: Sets the emergency stop status.
    - Parameters:
      - `self` - Mutable reference to the PriceOracle instance.
      - `stop` - Boolean value to set the emergency stop status.

11. `convert_switchboard_decimal(decimal: SwitchboardDecimal) -> Result<f64>`
    - Purpose: Converts a SwitchboardDecimal to an f64.
    - Parameters: `decimal` - The SwitchboardDecimal to convert.
    - Returns: `Result<f64>` - The converted f64 value or an error.
12. `PriceOracle::get_apy_from_feed(feed: &AccountLoader<AggregatorAccountData>) -> Result<f64>`
    - Purpose: Gets the APY from a Switchboard feed.
    - Parameters: `feed` - Reference to the Switchboard feed account.
    - Returns: `Result<f64>` - The APY from the feed or an error.

#### Error Codes

`OracleError` enum defines possible error types specific to the oracle functionality:

- `UnauthorizedAccess` - Unauthorized access attempt.
- `InvalidAssetType` - Invalid asset type.
- `DataNotAvailable` - Required data is not available.
- `InvalidAccountData` - Invalid account data.
- `InvalidPriceFeed` - Invalid price feed.
- `InvalidApyFeed` - Invalid APY feed.
- `PriceNotAvailable` - Price is not available.
- `ApyNotAvailable` - APY is not available.
- `InvalidDecimalConversion` - Invalid decimal conversion.
- `PriceChangeExceedsLimit` - Price change exceeds the allowed limit.
- `EmergencyStop` - Emergency stop is activated.
- `InvalidSwitchboardAccount` - Invalid Switchboard account.
- `StaleData` - Data is stale.
- `ExceedsConfidenceInterval` - Data exceeds the confidence interval.

## Usage

1. Ensure Rust and Solana CLI are installed.
2. Clone this repository.
3. Make sure your Solana keypair is correctly set up and accessible.
4. Run `anchor build` in the project root to build the program.
5. Run `anchor test` to execute all test cases.

## Troubleshooting

If you encounter errors related to missing Anchor macros or unable to find program files, try the following steps:

1. Ensure your Solana CLI is on the correct network (localnet for testing):
   ```
   solana config set --url localhost
   ```

2. Check the Solana and Anchor versions:
   ```
   solana --version
   anchor --version
   ```
   Make sure they are compatible with each other and with your project.

3. If problems persist, try manually deleting the `target` directory and `.anchor` directory (if it exists) and rebuilding:
   ```
   rm -rf target
   rm -rf .anchor
   anchor build
   ```

## Notes

- This program uses the Switchboard oracle to get real-time price and APY data.
- USDC price is fixed at $1.00.
- There's an emergency stop mechanism to prevent updates in case of abnormal conditions.
- Price changes exceeding 20% will trigger an error to prevent abnormal fluctuations.
- The `convert_switchboard_decimal` function is used to safely convert Switchboard's decimal representation to a standard f64 value.

## Running Tests

To run the tests for this project, follow these steps:

1. Ensure you have Solana CLI tools and Anchor installed on your system.

2. Navigate to the project root directory (flexxcash_bnpl) in your terminal.

3. Build the program:
   ```
   anchor build
   ```

4. Deploy the program to a local test validator:
   ```
   anchor deploy
   ```

5. Run the tests:
   ```
   anchor test
   ```

This will execute all the tests defined in the `tests/price_oracle.ts` file.

Note: Make sure you have a local Solana test validator running before executing the tests. If you encounter any issues, refer to the Troubleshooting section in this README.

## Contributing

Pull Requests are welcome to improve this project. Please ensure all tests pass before submitting.

## License

[MIT License](LICENSE)