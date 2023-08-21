# Rust Orderbook Implementation

This is an orderbook implementation written purely in Rust. It includes full support for market and limit orders, a FIFO matching engine, an account system, and a terminal UI that can be seen here:
<img width="1169" alt="Screenshot 2023-08-20 at 11 16 15 PM" src="https://github.com/AlpinYukseloglu/orderbook/assets/62043214/993c2c0c-d0f7-44dd-a23d-0b01512b1407">

## Features
  
- **Limit and Market Orders:** Create both limit orders (specifying a price) and market orders (executed immediately at the best available price).
  
- **FIFO Matching Engine:** Executes orders at the best available price. Orders on the same price level are processed on a first-in, first-out basis.
  
- **Account System:** Keeps track of balances, active and historical orders for each user.
  
- **Interactive Terminal UI:** A dynamic, real-time user interface right in your terminal, displaying the state of the order book, user balances, and more.
  
- **Macros for Test Data:** Macros to populate the orderbook with random distributions of orders to trade against.

## Terminal UI Commands

### IMPORTANT: these commands will only work in the orderbook CLI, which will be at the bottom of the terminal once `cargo run` successfully executes (see below for setup)

### General command template

```bash
[buy/sell] [osmo/usd] [market/limit] [amount] [price if limit]
```

### Example: placing a limit order to buy 10 OSMO at $0.40
```bash
buy osmo limit 10 0.40
```

### Example command for market selling 10,000 OSMO
```bash
sell osmo market 10000
```

## Macros

### Generate normal distribution of orders
Pressing `TAB` will run a macro that generates and places thousands of small orders that fall on roughly a normal distribution around the midpoint of the terminal screen. This can be run as many times as needed to get sufficient depth to trade against.

## Getting Started

### Prerequisites

Ensure you have Rust and Cargo installed on your system. If not, you can install them using [rustup](https://rustup.rs/).

### Installation

```bash
git clone https://github.com/yourusername/rust-orderbook.git
cd rust-orderbook
cargo build
```

### Usage

1. Launch the terminal UI:

    ```bash
    cargo run
    ```

2. Interact with the terminal UI to place orders, view balances, and more (using the commands described above):
    ```bash
    [buy/sell] [osmo/usd] [market/limit] [amount] [price if limit]
    ```
