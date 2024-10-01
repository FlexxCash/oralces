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
│           ├── price_oracle.rs
│           └── switchboard_utils.rs
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

## File Descriptions

### programs/oracles/src/lib.rs

This is the main program entry point, defining instructions for interacting with the price oracle.

#### Functions

1. `initialize(ctx: Context<Initialize>, switchboard_program_id: Pubkey) -> Result<()>`
   - Purpose: Initializes the price oracle accounts.

2. `update_prices_and_apys(ctx: Context<UpdatePricesAndApys>) -> Result<()>`
   - Purpose: Updates prices and APYs for all assets.

3. `update_sol_price(ctx: Context<UpdateSolPrice>) -> Result<()>`
   - Purpose: Updates the price for SOL.

4. `get_current_price(ctx: Context<GetPrice>, asset_type: AssetType) -> Result<()>`
   - Purpose: Gets the current price for a specified asset type.

5. `get_current_apy(ctx: Context<GetApy>, asset_type: AssetType) -> Result<()>`
   - Purpose: Gets the current APY for a specified asset type.

6. `set_emergency_stop(ctx: Context<SetEmergencyStop>, stop: bool) -> Result<()>`
   - Purpose: Sets the emergency stop status.

### programs/oracles/src/price_oracle.rs

This file implements the core logic of the price oracle.

#### Enums

`AssetType` enum defines supported asset types:
- JupSOL, VSOL, BSOL, MSOL, HSOL, JitoSOL, SOL

#### Structs

1. `PriceData`
   - Purpose: Stores price-related data.
   - Fields: price, last_price, last_update_time, apy

2. `PriceOracleHeader`
   - Purpose: Stores global oracle data.
   - Fields: last_global_update, emergency_stop, authority, switchboard_program_id, bump

3. `PriceOracleData`
   - Purpose: Stores price data for all assets.
   - Fields: price_data (array of PriceData), bump

#### Functions

1. `PriceOracle::initialize(...) -> Result<()>`
   - Purpose: Initializes the PriceOracle accounts.

2. `PriceOracle::update_prices_and_apys(...) -> Result<()>`
   - Purpose: Updates prices and APYs for all assets.

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

### programs/oracles/src/switchboard_utils.rs

This file contains utility functions for interacting with Switchboard oracles.

#### Constants

- `DEVNET_AGGREGATOR_PUBKEY`: Pubkey for the devnet aggregator
- `SOL_PRICE_AGGREGATOR_PUBKEY`: Pubkey for the SOL price aggregator

#### Structs

1. `SwitchboardResult`
   - Purpose: Stores a single Switchboard result.
   - Fields: value (f64)

2. `MultiAssetResult`
   - Purpose: Stores multiple asset results from Switchboard.
   - Fields: prices (array of f64), apys (array of f64)

#### Functions

1. `get_switchboard_result(...) -> Result<SwitchboardResult>`
   - Purpose: Retrieves a single result from a Switchboard feed.

2. `get_multi_asset_result(...) -> Result<MultiAssetResult>`
   - Purpose: Retrieves multiple asset results from a Switchboard feed.

3. `get_sol_price(...) -> Result<SwitchboardResult>`
   - Purpose: Retrieves the SOL price from a Switchboard feed.

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

## Running Tests

To run the tests for this project, follow these steps:

1. Ensure you have Solana CLI tools and Anchor installed.
2. Configure Solana CLI for local development: `solana config set --url localhost`
3. Start a local Solana test validator: `solana-test-validator`
4. In the project root, run: `anchor test`

The test suite in `tests/price_oracle.ts` covers the following scenarios:

- Initialization of the price oracle
- Updating prices and APYs for all assets
- Updating SOL price
- Getting current price and APY for assets
- Setting and checking emergency stop
- Handling unauthorized access attempts

## Notes

- This program uses the Switchboard oracle to get real-time price and APY data.
- There's an emergency stop mechanism to prevent updates in case of abnormal conditions.
- The program handles different data formats for regular assets and SOL price updates.
- The test suite covers a wide range of scenarios, including updates for all supported asset types and error cases.

## Contributing

Pull Requests are welcome to improve this project. Please ensure all tests pass before submitting.

## License

[MIT License](LICENSE)