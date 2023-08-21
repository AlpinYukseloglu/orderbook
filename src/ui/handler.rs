use std::cell::RefCell;
use std::rc::Rc;

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

        // Note: c is a generic char that interprets all alphanumeric characters
        KeyCode::Char(c) => {
            app.command_line.push(c); // Collect the character
        }
        KeyCode::Backspace => {
            app.command_line.pop(); // Remove the last character if there are any left
        }
        KeyCode::Enter => {
            // Process the input
            // handle_command(&input_buffer);
            app.command_line.clear(); // Reset the buffer for the next command
        }

        KeyCode::Char('o') => {
            // set up orderbook object
            let mut orderbook = orderbook::Orderbook::new(0);

            let account = Rc::new(RefCell::new(Account::new(0, AccountType::Individual)));
            account.borrow_mut().deposit(Currency::OSMO, 1000);
            account.borrow_mut().deposit(Currency::USD, 10000);

            // create a limit order to place using handle_order
            let mut order = order::Order::new(
                0,
                5,
                0,
                Rc::new(RefCell::new(Account::new(0, AccountType::Individual))),
                OrderType::Limit,
                OrderDirection::Bid,
                1000,
            );

            // place_and_process_order
            place_and_process_order(&mut orderbook, &mut order, app)?;
        }

        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}

// process and place order, which abstract everything form handle_order onwards
fn place_and_process_order(
    orderbook: &mut orderbook::Orderbook,
    order: &mut order::Order,
    app: &mut App,
) -> AppResult<()> {
    // In the Ok case, add "Order placed successfully" to the front of the app updates vector
    // In the err case, add "Error placing order: <error>" to the front of the app updates vector
    match orderbook.handle_order(order) {
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
                    order_quote_asset = orderbook.quote_asset();
                    order_base_asset = orderbook.base_asset();
                    order_price = *order.tick_id() as f64;
                }
                OrderDirection::Ask => {
                    order_quote_asset = orderbook.base_asset();
                    order_base_asset = orderbook.quote_asset();
                    let tick_id = *order.tick_id() as f64;
                    order_price = 1.0 / tick_id;
                }
            }
            
            match order.order_type() {
                // If limit order, we need to specify the price
                OrderType::Limit => {
                    app.updates.insert(
                        0,
                        format!(
                            "{} order successfully placed for {} {} at price {} {}.",
                            order.order_type().to_string(),
                            order.quantity(),
                            order_quote_asset.to_string(),
                            order_price,
                            order_base_asset.to_string(),
                        ),
                    );

                    insert_or_assign(&mut app.positions, *order.tick_id() as usize, *order.quantity());
                }

                // If market order, we don't need to specify a price
                OrderType::Market => {
                    app.updates.insert(
                        0,
                        format!(
                            "{} order for {} {} successfully placed.",
                            order.order_type().to_string(),
                            order.quantity(),
                            order_quote_asset.to_string(),
                        ),
                    );
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