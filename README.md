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
- bytemuck = "1.13.1"
- rust_decimal = "1.26.1"
- solana-program = { version = ">=1.16, <1.17" }
- superslice = "1"

## File Descriptions

### programs/oracles/src/lib.rs

This is the main program entry point, defining instructions for interacting with the price oracle.

#### Constants

- `PRICE_CHANGE_THRESHOLD: f64 = 0.20` - Defines the maximum allowed price change (20%) before triggering a warning.

#### Functions

1. `initialize(ctx: Context<Initialize>, switchboard_program_id: Pubkey) -> Result<()>`
   - Purpose: Initializes the price oracle accounts.
   - Parameters: 
     - `ctx` - Context containing accounts needed for initialization.
     - `switchboard_program_id` - The Pubkey of the Switchboard program.
   - Returns: `Ok(())` on success.

2. `update_price_and_apy(ctx: Context<UpdatePriceAndApy>, asset_type: AssetType) -> Result<()>`
   - Purpose: Updates both the price and APY for a specified asset type.
   - Parameters:
     - `ctx` - Context containing accounts needed for update.
     - `asset_type` - The type of asset to update.
   - Returns: `Ok(())` on success.

3. `update_sol_price(ctx: Context<UpdateSolPrice>) -> Result<()>`
   - Purpose: Updates the price for SOL.
   - Parameters:
     - `ctx` - Context containing accounts needed for SOL price update.
   - Returns: `Ok(())` on success.

4. `update_switchboard_program_id(ctx: Context<UpdateSwitchboardProgramId>, new_program_id: Pubkey) -> Result<()>`
   - Purpose: Updates the Switchboard program ID.
   - Parameters:
     - `ctx` - Context containing accounts needed for update.
     - `new_program_id` - The new Switchboard program ID.
   - Returns: `Ok(())` on success.

#### Structs

1. `Initialize<'info>`
   - Purpose: Defines accounts needed for initialization.
   - Fields:
     - `header: Account<'info, PriceOracleHeader>` - The price oracle header account.
     - `data: Account<'info, PriceOracleData>` - The price oracle data account.
     - `authority: Signer<'info>` - The authority initializing the accounts.
     - `system_program: Program<'info, System>` - The system program.

2. `UpdatePriceAndApy<'info>`
   - Purpose: Defines accounts needed for updating price and APY.
   - Fields:
     - `header: Account<'info, PriceOracleHeader>` - The price oracle header account.
     - `data: Account<'info, PriceOracleData>` - The price oracle data account.
     - `oracle_feed: AccountLoader<'info, AggregatorAccountData>` - The Switchboard oracle feed.
     - `authority: Signer<'info>` - The authority allowed to update.

3. `UpdateSolPrice<'info>`
   - Purpose: Defines accounts needed for updating SOL price.
   - Fields:
     - `header: Account<'info, PriceOracleHeader>` - The price oracle header account.
     - `data: Account<'info, PriceOracleData>` - The price oracle data account.
     - `oracle_feed: AccountLoader<'info, AggregatorAccountData>` - The Switchboard oracle feed.
     - `authority: Signer<'info>` - The authority allowed to update.

4. `UpdateSwitchboardProgramId<'info>`
   - Purpose: Defines accounts needed for updating Switchboard program ID.
   - Fields:
     - `header: Account<'info, PriceOracleHeader>` - The price oracle header account.
     - `authority: Signer<'info>` - The authority allowed to update.

### programs/oracles/src/price_oracle.rs

This file implements the core logic of the price oracle.

#### Enums

`AssetType` enum defines supported asset types:
- JupSOL, MSOL, VSOL, BSOL, HSOL, JitoSOL, SOL

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
     - `switchboard_program_id: Pubkey` - Switchboard program ID.
     - `bump: u8` - PDA bump.

3. `PriceOracleData`
   - Purpose: Stores price data for all assets.
   - Fields:
     - `price_data: Vec<PriceData>` - Vector of price data for each asset.
     - `asset_types: Vec<AssetTypeWrapper>` - Vector of asset types.
     - `bump: u8` - PDA bump.

#### Functions

1. `PriceOracle::initialize(...) -> Result<()>`
   - Purpose: Initializes the PriceOracle accounts.

2. `PriceOracle::update_price_and_apy(...) -> Result<()>`
   - Purpose: Updates both price and APY for a given asset type.

3. `PriceOracle::update_sol_price(...) -> Result<()>`
   - Purpose: Updates the SOL price.

4. `PriceOracle::get_current_price(...) -> Result<f64>`
   - Purpose: Gets the current price for a given asset type.

5. `PriceOracle::get_current_apy(...) -> Result<f64>`
   - Purpose: Gets the current APY for a given asset type.

6. `PriceOracle::is_emergency_stopped(...) -> bool`
   - Purpose: Checks if the emergency stop is activated.

7. `PriceOracle::set_emergency_stop(...)`
   - Purpose: Sets the emergency stop status.

#### Error Codes

`OracleError` enum defines possible error types:

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
- `MaxAssetsReached` - Maximum number of assets reached.
- `AssetNotFound` - Asset not found.
- `InvalidIndex` - Invalid index.
- `ClockUnavailable` - System clock is unavailable.
- `InvalidSwitchboardProgram` - Invalid Switchboard program.
- `InvalidSwitchboardData` - Invalid data from Switchboard.

## Switchboard Data Format

The Switchboard oracle provides price and APY data in the following format:

```json
{
  "result": "81.14553583522887934",
  "results": [
    "163.582759",
    "0.07883967045775868",
    "162.212232",
    "0.06710272249494009",
    "181.854954",
    "0.06659877596838777",
    "191.787336",
    "0.07353018328401717",
    "162.237243",
    "0.07424004899565666",
    "179.229791",
    "0.07153565738789942"
  ],
  "version": "RC_09_16_24_18_54"
}
```

The "results" array contains price and APY data for different assets in the following order:
1. JupSOL price
2. JupSOL APY
3. vSOL price
4. vSOL APY
5. bSOL price
6. bSOL APY
7. mSOL price
8. mSOL APY
9. HSOL price
10. HSOL APY
11. JitoSOL price
12. JitoSOL APY

For SOL price, a separate Switchboard feed is used with the following format:

```json
{
  "result": "156.55828500000000000000000000",
  "results": [
    "156.56328000000000000000",
    "task 21614352 panicked",
    "157.19",
    "156.55329000000000000000000000",
    "156.527665650000000000000000"
  ],
  "version": "RC_09_16_24_18_54"
}
```

The SOL price is taken from the "result" field in this case.

## Usage

1. Ensure Rust and Solana CLI are installed.
2. Clone this repository.
3. Make sure your Solana keypair is correctly set up and accessible.
4. Run `anchor build` in the project root to build the program.
5. Run `anchor test` to execute all test cases.

## Troubleshooting

If you encounter errors related to missing Anchor macros or unable to find program files, try the following steps:

1. Ensure your Solana CLI is on the correct network (localnet for testing):

## Running Tests

To run the tests for this project, follow these detailed steps:

1. Ensure you have Solana CLI tools and Anchor installed on your system:
   ```
   solana --version
   anchor --version
   ```
   If not installed, follow the official documentation to install them.

2. Clone this repository and navigate to the project root directory:
   ```
   git clone <repository-url>
   cd flexxcash_bnpl
   ```

3. Install project dependencies:
   ```
   npm install
   ```

4. Ensure your Solana CLI is configured for local development:
   ```
   solana config set --url localhost
   ```

5. Start a local Solana test validator in a separate terminal window:
   ```
   solana-test-validator
   ```

6. Build the Anchor program:
   ```
   anchor build
   ```

7. Deploy the program to the local test validator:
   ```
   anchor deploy
   ```

8. Run the tests:
   ```
   anchor test
   ```

This will execute all the tests defined in the `tests/price_oracle.ts` file.

### Test Coverage

The test suite in `tests/price_oracle.ts` covers the following scenarios:

- Initialization of the price oracle
- Updating price and APY for various asset types (JupSOL, VSOL, etc.)
- Updating SOL price
- Getting current price and APY for assets
- Setting and checking emergency stop
- Handling unauthorized access attempts
- Updating multiple assets simultaneously
- Updating Switchboard program ID

These tests ensure that all major functionalities of the price oracle are working as expected, including edge cases and error handling.

### Troubleshooting Test Issues

If you encounter any issues while running the tests, try the following:

1. Ensure the Solana test validator is running and responsive.

2. If you get errors about account or program not found, try resetting the test validator and redeploying:
   ```
   solana-test-validator --reset
   anchor deploy
   ```

3. Check that your Anchor.toml file has the correct program ID and cluster configuration.

4. If you're getting TypeScript compilation errors, ensure your node_modules are up to date:
   ```
   npm install
   ```

5. For Anchor-specific errors, consult the Anchor documentation or try cleaning and rebuilding:
   ```
   anchor clean
   anchor build
   anchor deploy
   ```

6. If tests are failing due to timeout issues, you can increase the timeout in the test script within package.json.

Remember to check the console output for specific error messages, which can provide clues about what might be going wrong.

## Notes

- This program uses the Switchboard oracle to get real-time price and APY data.
- There's an emergency stop mechanism to prevent updates in case of abnormal conditions.
- Price changes exceeding 20% will trigger a warning log.
- The `convert_switchboard_decimal` function is used to safely convert Switchboard's decimal representation to a standard f64 value.
- The program handles different data formats for regular assets and SOL price updates.
- The test suite covers a wide range of scenarios, including updates for all supported asset types and error cases.

## Contributing

Pull Requests are welcome to improve this project. Please ensure all tests pass before submitting. When making changes, make sure to update or add relevant tests in the `tests/price_oracle.ts` file to maintain comprehensive test coverage.

## License

[MIT License](LICENSE)