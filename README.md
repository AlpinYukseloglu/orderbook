# Rust Orderbook Implementation

## Features
  
- **Limit and Market Orders:** Create both limit orders (specifying a price) and market orders (executed immediately at the best available price).
  
- **FIFO Matching Engine:** Executes orders at the best available price. Orders on the same price level are processed on a first-in, first-out basis.
  
- **Account System:** Keeps track of balances, active and historical orders for each user.
  
- **Interactive Terminal UI:** A dynamic, real-time user interface right in your terminal, displaying the state of the order book, user balances, and more.
  
- **Macros for Test Data:** Macros to populate the orderbook with random distributions of orders to trade against.

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

2. Interact with the terminal UI to place orders, view balances, and more.

![e](https://hackmd.io/_uploads/Hkgywl-6h.png)
