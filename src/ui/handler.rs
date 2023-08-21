use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use rand::prelude::*;
use rand_distr::{Distribution, Normal};

use crate::bank::account::{Account, AccountType};
use crate::bank::currency::Currency;
use crate::ui::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
// import book and set up Orderbook object
use crate::book::orderbook;
// import order
use crate::book::order::{self, OrderDirection, OrderType};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC`
        KeyCode::Esc => {
            app.quit();
        }

        KeyCode::Tab => {
            generate_normal_distribution_orders(app, 1, 40)?;
        }

        // Note: c is a generic char that interprets all alphanumeric characters
        KeyCode::Char(c) => {
            app.command_line.push(c); // Collect the character
        }
        KeyCode::Backspace => {
            app.command_line.pop(); // Remove the last character if there are any left
        }
        KeyCode::Enter => {
            // Process the input
            handle_command(app)?;
            // wait 50 ms
            std::thread::sleep(std::time::Duration::from_millis(50));
            app.command_line.clear();
        }

        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}

// handle command function (takes in orderbook, account, app, and command string)
// "buy OSMO": bid order direction
// "sell OSMO": ask order direction
// "buy USD": ask order direction
// "sell USD": bid order direction
// if the third argument is "limit" (case insensitive), order type is limit
// if the third argument is "market" (case insensitive), order type is market
// the fourth argument is the quantity of the order
// only require the fifth argument if the order type is limit, and use the price times 10 as the tick_id
// Use this information to build an Order object and pass it to place_and_process_order
fn handle_command(app: &mut App) -> AppResult<()> {
    let tokens: Vec<&str> = app.command_line.split_whitespace().collect();

    if tokens.len() < 4 {
        // Invalid command format
        app.command_line = "Invalid command format: ".to_string() + &(tokens.join(" "));
        return Ok(());
    }

    let order_direction = match tokens[0].to_lowercase().as_str() {
        "buy" if tokens[1].eq_ignore_ascii_case("OSMO") => OrderDirection::Bid,
        "sell" if tokens[1].eq_ignore_ascii_case("OSMO") => OrderDirection::Ask,
        "buy" if tokens[1].eq_ignore_ascii_case("USD") => OrderDirection::Ask,  // Note the inversion
        "sell" if tokens[1].eq_ignore_ascii_case("USD") => OrderDirection::Bid,  // Note the inversion
        _ => {
            app.command_line = "Unsupported command format".to_string();
            return Ok(());
        }
    };

    let order_type = match tokens[2].to_lowercase().as_str() {
        "limit" => OrderType::Limit,
        "market" => OrderType::Market,
        _ => {
            app.command_line = "Unsupported order type".to_string();
            return Ok(());
        }
    };

    let quantity: u64 = match tokens[3].parse() {
        Ok(q) => q,
        Err(_) => {
            app.command_line = "Failed to parse quantity".to_string();
            return Ok(());
        }
    };

    let tick_id = if let OrderType::Limit = order_type {
        if tokens.len() < 5 {
            app.command_line = "Missing price argument for limit order".to_string();  // Missing price argument for limit order
            return Ok(());
        }
        
        let price: f64 = match tokens[4].parse() {
            Ok(p) => p,
            Err(_) => {
                app.command_line = "Failed to parse price".to_string();
                return Ok(());
            }
        };
        
        (price * 10.0).trunc() as u64
    } else {
        0  // Default value if not a limit order
    };

    // Here I am assuming order_id, book_id are default set as 0. Adjust as necessary.
    let mut order = order::Order::new(
        0,
        tick_id,
        0,
        app.user_account.clone(),
        order_type,
        order_direction,
        quantity,
    );

    place_and_process_order(&mut order, app)?;
    
    Ok(())
}


// Place order and wire up result to UI
fn place_and_process_order(
    order: &mut order::Order,
    app: &mut App,
) -> AppResult<()> {
    // In the Ok case, add "Order placed successfully" to the front of the app updates vector
    // In the err case, add "Error placing order: <error>" to the front of the app updates vector
    match app.session_book.handle_order(order) {
        Ok(_) => {
            // now also add what kind of order and for how much e.g. "Limit for 1000 OSMO (quote asset) placed successfully at price (tick_id)"
            // A general framing for this is "{OrderType} for {Order.quantity()} {orderbook.quote_asset()} placed successfully"
            
            // if order was a bid, then the order was for OSMO (quote asset) and the price was the tick_id USD
            // if order was an ask, then the order was for USD (base asset) and the price was 1/tick_id per OSMO
            let order_quote_asset;
            let order_base_asset;
            let order_price: f64;

            match order.order_direction() {
                OrderDirection::Bid => {
                    order_quote_asset = app.session_book.quote_asset();
                    order_base_asset = app.session_book.base_asset();
                    order_price = *order.tick_id() as f64;
                }
                OrderDirection::Ask => {
                    order_quote_asset = app.session_book.base_asset();
                    order_base_asset = app.session_book.quote_asset();
                    let tick_id = *order.tick_id() as f64;
                    order_price = 1.0 / tick_id;
                }
            }
            
            match order.order_type() {
                // If limit order, we need to specify the price
                OrderType::Limit => {
                    app.updates.push(
                        format!(
                            "{} order successfully placed for {} {} at price {} {}.",
                            order.order_type().to_string(),
                            order.quantity(),
                            order_quote_asset.to_string(),
                            order_price / 10.0,
                            order_base_asset.to_string(),
                        ),
                    );

                    // iterate through all active ticks in orderbook tick tree and update
                    let mut new_positions = Vec::new();
                    for (key, value) in app.session_book.ticks().range(..) {
                        insert_or_assign(&mut new_positions, *key as usize, *value.total_orders());
                    }
                    app.positions = new_positions;
                }

                // If market order, we don't need to specify a price
                OrderType::Market => {
                    app.updates.push(
                        format!(
                            "{} order for {} {} successfully placed. Order will be filled for however much {} is available at the best price.",
                            order.order_type().to_string(),
                            order.quantity(),
                            order_quote_asset.to_string(),
                            order_quote_asset.to_string(),
                        ),
                    );

                    let mut new_positions = Vec::new();
                    for (key, value) in app.session_book.ticks().range(..) {
                        insert_or_assign(&mut new_positions, *key as usize, *value.total_orders());
                    }
                    app.positions = new_positions;
                }
            }
        }
        Err(e) => {
            app.updates
                .insert(0, format!("Error placing order: {}", e));
        }
    }
    Ok(())
}

fn insert_or_assign(vec: &mut Vec<u64>, index: usize, value: u64) {
    if vec.len() <= index {
        vec.resize_with(index + 1, Default::default); // This will fill in gaps with 0
    }
    vec[index] = value;
}

// Generates a normal distribution of orders
fn generate_normal_distribution_orders(app: &mut App, min_tick: u64, max_tick: u64) -> AppResult<()> {
    // set up the normal distribution
    let mid_point = (max_tick as f64 + min_tick as f64) / 2.0;
    let standard_deviation = (max_tick - min_tick) as f64 / 6.0; // Roughly 99.7% of data will be within min_tick and max_tick
    let normal = Normal::new(mid_point, standard_deviation).unwrap();

    // bot account
    let acc = Rc::new(RefCell::new(Account::new(0, AccountType::Individual)));
    acc.borrow_mut().deposit(Currency::OSMO, 10000000000);
    acc.borrow_mut().deposit(Currency::USD, 10000000000);

    // generate the orders
    for _ in 0..20000 { // replace number_of_orders with your desired number
        let tick_id = normal.sample(&mut thread_rng()).round() as u64;
        if tick_id < min_tick || tick_id > max_tick || tick_id == mid_point as u64 {
            continue; // Skip this order if the tick_id falls outside our range
        }

        let quantity = 1; // Fixed quantity per order

        // Set the order type to Limit
        let order_type = OrderType::Limit;

        // Determine the order direction based on the tick_id relative to the midpoint
        let order_direction = if tick_id < mid_point as u64 {
            OrderDirection::Bid
        } else {
            OrderDirection::Ask
        };

        let mut order = order::Order::new(
            0,
            tick_id,
            0,
            acc.clone(),
            order_type,
            order_direction,
            quantity,
        );

        place_and_process_order(&mut order, app)?;
    }

    Ok(())
}
