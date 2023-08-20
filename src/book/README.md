# Orderbook Operations

The `book` folder contains all operations related to orderbooks. Here is a brief breakdown of what each file does:
1. `orderbook.rs`: Contains the `Orderbook` struct and all functions it directly implements. This includes creating new orderbooks and high level order operations that then get routed to the appropriate tick to be processed.
2. `tick.rs`: Defines tick structs, including tick initialization, adding orders to ticks, filling orders on ticks etc.
3. `order.rs`: Defines the `Order` struct, enums for order types.
4. `query.rs`: An interface layer for querying the orderbook. This is used primarily by the terminal frontend to fetch information about the orderbook in a processed way.