# Stylus Interest Calculator

This project is a smart contract for interest calculation that runs on the Arbitrum Stylus network. The main purpose of this project is to demonstrate and test the performance improvements and optimizations between Stylus SDK versions v0.6.0 and v0.8.3.

## Project Purpose

This project serves as a benchmark to test and compare the improvements in the Stylus SDK from version v0.6.0 to v0.8.3. The key improvements tested include:

- Cache optimization for better performance
- Reduced gas consumption
- Improved execution time
- Enhanced calculation accuracy
- Better memory management

## Features

- Simple and compound interest calculation
- Principal, interest rate, and period configuration
- Cache-optimized calculations (v0.8.3)
- Interest accumulation tracking

## Versions

Two versions are provided:

- v0.6.0: Basic interest calculation functionality
- v0.8.3: Improved version with cache optimization

## Installation

1. Install Rust (1.70.0 or higher recommended)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install Docker

```bash
# For macOS
brew install docker

# For Ubuntu
sudo apt-get update
sudo apt-get install docker.io
```

3. Install Stylus CLI

```bash
cargo install cargo-stylus
```

4. Build the project

```bash
cd counter_v0_8_3  # or counter_v0_6_0
```

## Usage

1. Build and check the contract (requires Docker)

```bash
cargo stylus check --endpoint YOUR_RPC_ENDPOINT
```

2. Export ABI

```bash
mkdir -p output
cargo stylus export-abi --json 2>/dev/null | grep -v "=======" | grep -v "Contract JSON ABI" | grep -v "^$" > output/abi.json
```

3. Deploy the contract (requires Docker)

```bash
cargo stylus deploy --endpoint YOUR_RPC_ENDPOINT --private-key YOUR_PRIVATE_KEY
```

4. Initialize the contract

```rust
calculator.initialize()
```

5. Set principal amount

```rust
calculator.set_principal(amount)
```

6. Set interest rate (default: 5.5% = 550)

```rust
calculator.set_rate(rate)
```

7. Set period (in years)

```rust
calculator.set_period(period)
```

8. Set compound/simple interest (1: compound, 0: simple)

```rust
calculator.set_compound(is_compound)
```

## Performance Comparison

The v0.8.3 version provides the following improvements through cache optimization:

- Reduced gas consumption
- Faster execution time
- More accurate interest calculations
- Better memory efficiency

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.
