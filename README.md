<div align="center">
    <img src="https://user-images.githubusercontent.com/107150702/179010980-d4e2bd7d-3abf-4e73-b3c3-d104ee9b780f.png" alt="Roloi logo" height="150"/>

<h1 align="center">Streams Smart Contract</h1>

[![Built with ink!](https://raw.githubusercontent.com/paritytech/ink/master/.images/badge_flat.svg)](https://github.com/paritytech/ink)
![Twitter Follow](https://img.shields.io/twitter/follow/RoloiMoney?style=social)

</div>

> Roloi is a **payment customization platform** that provides **money streams, recurring and conditional payments**. Roloi converts money into a **fluid asset** that can flow through a blockchain network. **In a dynamic financial world, dynamic payments are necessary.**

## Environment setup

To compile the smart contract, Rust and Cargo are required. Here is an [installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html).

[`cargo-contract`](https://github.com/paritytech/cargo-contract) is required too. Install it using this command:

```
cargo install cargo-contract --force
```

## Run tests off-chain

To run the tests off-chain, execute this command:

```
cargo +nightly contract test
```

## Generate technical documentation

To generate the technical documentation, execute this command:

```
cargo doc
```

## Build smart contract

To compile the smart contract and generates the optimized WebAssembly bytecode, the metadata and bundles, execute this command:

```
cargo +nightly contract build
```

## Upload & Instantiate

Open the [Substrate Contracts-UI](https://contracts-ui.substrate.io).

Choose a chain in the dropdown placed in the top section of the left menu.

Follow the [ink! guide](https://ink.substrate.io/getting-started/deploy-your-contract) to upload and instantiate the smart contract.

## Data Model

### Stream

```rust
pub struct Stream {
    pub payer: AccountId,
    pub recipient: AccountId,
    pub original_balance: u128,
    pub current_balance: u128,
    pub start_date: u64,
    pub end_date: u64
}
```

### Storage

```rust
#[ink(storage)]
#[derive(SpreadAllocate)]
pub struct StreamsContract {
    owner: AccountId,
    next_stream_id: u64,
    streams: Mapping<u64, Stream>
}
```

## Messages

### Create Stream

> Creates a token stream from the sender to the specified recipient setting the end date or the duration.

```rust
create_stream(
    recipient: AccountId,
    end_date: Option<u64>,
    duration: Option<u64>,
) -> Result<u64, ContractError>
```

Parameters:

- `recipient`: The recipient wallet address of the stream.
- `end_date`: The end date of the stream measured in seconds. If not specified, the stream will be created with the duration.
- `duration`: The duration of the stream measured in seconds. If not specified, the stream will be created with the end date.
- **Transaction funds:** The amount of funds to be transferred to the recipient through the stream.

Returns:

- The created stream ID.

Examples:

- Using `end_date`: 01/01/2024 01:00:00 GMT

```rust
    create_stream(
        "5FPE9bPa2yx8dhREN6iZ3PhAJqhbY87Gfg4ELJiRt8P5xUqN",
        Some(1704070800),
        None
    );
```

- Using `duration`: 5 minutes

```rust
    create_stream(
        "5FPE9bPa2yx8dhREN6iZ3PhAJqhbY87Gfg4ELJiRt8P5xUqN",
        None,
        Some(300)
    );
```

### Recipient Withdraw

> Withdraws tokens from a stream. The recipient can specify the expected amount of tokens or withdraw all the available balance.

```rust
recipient_withdraw(
    stream_id: u64,
    withdrawal_amount: Option<u128>,
) -> Result<u128, ContractError>
```

Parameters:

- `stream_id`: The stream ID.
- `withdrawal_amount`: The amount of tokens to be withdrawn. If not specified, all the available balance will be withdrawn.

Returns:

- The amount of tokens withdrawn.

Examples:

- Specifying `withdrawal_amount`: 1000

```rust
recipient_withdraw(1, Some(1000));
```

- Without specifying `withdrawal_amount`

```rust
recipient_withdraw(1, None);
```

### Get Stream by ID

> Returns a stream by its ID.

```rust
get_stream_by_id(
    stream_id: u64
) -> Result<Stream, ContractError>
```

Parameters:

- `stream_id`: The expected stream ID.

Returns:

- The expected stream.

Example:

```rust
get_stream_by_id(1);
```
