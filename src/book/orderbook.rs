use super::order::*;
use super::tick::Tick;
use crate::bank::account::*;
use crate::bank::currency::*;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::process;

struct Orderbook {
    book_id: u64,
    quote_asset: Currency,
    base_asset: Currency,
    next_bid_tick: u64,
    next_ask_tick: u64,
    ticks: BTreeMap<u64, Tick>,
    cancellation_map: HashMap<u64, u64>,
}

impl Orderbook {
    fn new(book_id: u64) -> Orderbook {
        // Initialize min and max ticks in cancellation map.
        // This is to track whether the book has run out of ticks when doing market orders.
        let mut cancellation_map = HashMap::new();
        cancellation_map.insert(u64::MIN, u64::MIN);

        // We default to an OSMO/USD pair for now. This can be generalized to more assets later.
        Orderbook {
            book_id: book_id,
            quote_asset: Currency::OSMO,
            base_asset: Currency::USD,
            next_bid_tick: u64::MAX,
            next_ask_tick: u64::MIN,
            ticks: BTreeMap::new(),
            cancellation_map: cancellation_map,
        }
    }

    fn handle_order(&mut self, order: &mut Order) {
        // TODO: subtract balance here with a check that returns error/result
        
        match order.order_type() {
            OrderType::Market => {
                self.run_market_order(order)
            }
            OrderType::Limit => {
                self.run_partial_or_full_limit(order);
            }
        }
    }

    fn cancel_order(&mut self, order_id: u64) {}

    // For T existing initialized ticks, do a log(T) search/insert for the tick_id in our BTreeMap.
    fn get_or_init_tick_in_tree(&mut self, tick_id: u64) -> &mut Tick {
        self.ticks.entry(tick_id).or_insert(Tick::new(tick_id))
    }

    // Place limit on specified tick and properly handle error if there is an issue.
    fn run_place_limit(&mut self, order: Order) {
        let tick_id = *order.tick_id();
        let tick = self.get_or_init_tick_in_tree(tick_id);
        if let Err(e) = tick.place_limit(order) {
            println!("Problem placing limit order: {}", e);
            process::exit(1);
        }
    }

    // Implement market bid abstraction that takes in a start tick and fills ticks as bids
    fn run_market_bid(&mut self, order: &mut Order, end_tick: u64, quantity: u64) -> u64 {
        let mut remaining_quantity = quantity;
        let mut to_remove = Vec::new();
    
        // Define scope to borrow self.ticks as mutable in scope.
        // When this scope ends, the borrow is dropped, letting us go back through to remove empty ticks.
        {
            // Reverse iterate, inclusive of start tick
            let mut tick_iter = self.ticks.range_mut(..=self.next_bid_tick);
    
            while remaining_quantity > 0 && self.next_bid_tick != u64::MIN {
                if let Some((tick_id, tick)) = tick_iter.next_back() {
                    // If the next tick is below our start tick, we cut off the market bid process.
                    if *tick_id <= end_tick {
                        self.next_bid_tick = end_tick;
                        break;
                    } else {
                        self.next_bid_tick = *tick_id;
                    }
    
                    // Fill the tick and update remaining quantity
                    let pre_fill_remaining = remaining_quantity;
                    remaining_quantity = tick.fill_tick(remaining_quantity);
                    let filled_quantity = pre_fill_remaining - remaining_quantity;

                    // Distribute purchased assets to order owner
                    order.distribute_filled_assets(filled_quantity, *tick.tick_id());
    
                    // If tick was fully filled, set to remove it from the book
                    if tick.orders().len() == 0 {
                        to_remove.push(*tick_id);
                    }
                }
            }
        }
    
        // Remove empty ticks from the book
        for tick_id in to_remove {
            self.ticks.remove(&tick_id);
        }

        return remaining_quantity;
    }
    

    // Implement market ask abstraction that takes in a start tick and fills ticks as asks
    fn run_market_ask(&mut self, order: &mut Order, end_tick: u64, quantity: u64) -> u64 {
        let mut remaining_quantity = quantity;
        let mut to_remove = Vec::new();
    
        // Define scope to borrow self.ticks as mutable in scope.
        // When this scope ends, the borrow is dropped, letting us go back through to remove empty ticks.
        {
            let mut tick_iter = self.ticks.range_mut(self.next_ask_tick..);
    
            while remaining_quantity > 0 && self.next_ask_tick != u64::MAX {
                if let Some((tick_id, tick)) = tick_iter.next() {
                    // If next tick is at or past our end tick, we cut off the market ask process.
                    if *tick_id >= end_tick {
                        self.next_ask_tick = end_tick;
                        break;
                    } else {
                        self.next_ask_tick = *tick_id;
                    }

                    // Fill the tick and update remaining quantity
                    let pre_fill_remaining = remaining_quantity;
                    remaining_quantity = tick.fill_tick(remaining_quantity);
                    let filled_quantity = pre_fill_remaining - remaining_quantity;

                    // Distribute purchased assets to order owner
                    order.distribute_filled_assets(filled_quantity, *tick.tick_id());
                    
                    // If tick was fully filled, set to it from the book
                    if tick.orders().len() == 0 {
                        to_remove.push(*tick_id);
                    }
                }
            }
        }
    
        // Remove empty ticks from book
        for tick_id in to_remove {
            self.ticks.remove(&tick_id);
        }

        return remaining_quantity;
    }

    // handle partial limits
    fn run_partial_or_full_limit(&mut self, order: &mut Order) {
        let tick_id = *order.tick_id();
                let mut remaining_quantity = *order.quantity();
                match order.order_direction() {
                    OrderDirection::Bid => {
                        // If the bid is past the lowest ask, immediately fill the appropriate portion of the order.
                        if tick_id > self.next_ask_tick {
                            remaining_quantity = self.run_market_bid(order, tick_id, remaining_quantity)
                        }
                        
                        if remaining_quantity > 0 && self.next_ask_tick != u64::MAX {
                            order.set_quantity(remaining_quantity);

                            // We clone here because we don't own `order`, just a mutable reference to it.
                            let cloned_order = order.clone();
                            self.run_place_limit(cloned_order);
                        }
                    }
                    OrderDirection::Ask => {
                        // If the ask is past the highest bid, immediately fill the appropriate portion of the order.
                        if tick_id < self.next_bid_tick {
                            remaining_quantity = self.run_market_ask(order, tick_id, remaining_quantity)
                        }

                        if remaining_quantity > 0 && self.next_bid_tick != u64::MIN {
                            order.set_quantity(remaining_quantity);

                            // We clone here because we don't own `order`, just a mutable reference to it.
                            let cloned_order = order.clone();
                            self.run_place_limit(cloned_order);
                        }
                    }
                }
    }

    fn run_market_order(&mut self, order: &mut Order) {
        // In both cases, we let the return value drop quietly. This is the equivalent of not erroring if the market runs out of ticks,
        // which is appropriate behavior for a market order that is large enough to clear the book.
        let remaining_quantity = *order.quantity();
        match order.order_direction() {
            OrderDirection::Bid => {
                self.run_market_bid(order, u64::MIN, remaining_quantity);
            }
            OrderDirection::Ask => {
                self.run_market_ask(order, u64::MAX, remaining_quantity);
            }
        }
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bank::currency::Currency;

    // Test helper that creates a specified number of orders of equal quantity on the passed in tick
    fn create_limit_orders(book: &mut Orderbook, tick_id: &mut u64, num_orders: u64, quantity: u64, order_direction: &OrderDirection) {
        for i in 0..num_orders {
            let order = Order::new(
                i,
                *tick_id,
                0,
                Account::new(i, AccountType::Individual),
                OrderType::Limit,
                *order_direction,
                quantity,
            );

            let tick = book.get_or_init_tick_in_tree(*tick_id);

            if let Err(e) = tick.place_limit(order) {
                println!("Problem placing limit order: {}", e);
                process::exit(1);
            }

        }
    }

    // implement test case where the order book's next ask tick is 10, and there are orders on ticks 10 to 15 (using helper above)
    #[test]
    fn test_run_market_ask() {
        let mut book = Orderbook::new(0);

        // set next ask tick on book to tick 10
        book.next_ask_tick = 10;

        // create orders on tick 10 (next ask tick)
        create_limit_orders(&mut book, &mut 10, 3, 100, &OrderDirection::Ask);

        // create orders on ticks 13, 14, and 21 to add more depth on the ask side
        create_limit_orders(&mut book, &mut 13, 3, 100, &OrderDirection::Ask);
        create_limit_orders(&mut book, &mut 14, 3, 100, &OrderDirection::Ask);
        create_limit_orders(&mut book, &mut 21, 3, 100, &OrderDirection::Ask);

        // run market ask for 1000 quantity
        // since market depth is 1200 (four ticks with 300 each), this should fill up to tick 21
        let mut order = Order::new(
            13,
            0,
            0,
            Account::new(0, AccountType::Individual),
            OrderType::Market,
            OrderDirection::Ask,
            1000,
        );
        book.run_market_ask(&mut order, u64::MAX, 1000);

        // ticks 10, 13, and 14 should all be emptied and removed from the book
        assert!(!book.ticks.contains_key(&10));
        assert!(!book.ticks.contains_key(&13));
        assert!(!book.ticks.contains_key(&14));

        // tick 21 should still be in the book and have 200 quantity left
        assert!(book.ticks.contains_key(&21));

        // assert with total liq on tick once tracking is implemented
        // assert_eq!(book.ticks.get(&21).unwrap().quantity(), 200);

        // next ask tick should be updated to 21
        assert_eq!(book.next_ask_tick, 21);

        // We expect the USD balance to be equal to the quantity filled at each tick times the prices at each tick
        assert_eq!(order.owner().balance(Currency::USD), 300 * (10 + 13 + 14) + 100 * 21);
    }

    // implement a similar run market ask test but with a specified end tick at 15
    #[test]
    fn test_run_market_ask_with_end_tick() {
        let mut book = Orderbook::new(0);

        // set next ask tick on book to tick 10
        book.next_ask_tick = 10;

        // create orders on tick 10 (next ask tick)
        create_limit_orders(&mut book, &mut 10, 3, 100, &OrderDirection::Ask);

        // create orders on ticks 13, 14, and 21 to add more depth on the ask side
        create_limit_orders(&mut book, &mut 13, 3, 100, &OrderDirection::Ask);
        create_limit_orders(&mut book, &mut 14, 3, 100, &OrderDirection::Ask);
        create_limit_orders(&mut book, &mut 21, 3, 100, &OrderDirection::Ask);

        // run market ask for 1000 quantity
        // since market depth is 1200 (four ticks with 300 each), this should fill up to tick 15
        let mut order = Order::new(
            13,
            0,
            0,
            Account::new(0, AccountType::Individual),
            OrderType::Market,
            OrderDirection::Ask,
            1000,
        );
        book.run_market_ask(&mut order, 21, 1000);

        // ticks 10, 13, and 14 should all be emptied and removed from the book
        assert!(!book.ticks.contains_key(&10));
        assert!(!book.ticks.contains_key(&13));
        assert!(!book.ticks.contains_key(&14));

        // tick 21 should still be in the book and remain untouched, as we stopped filling before processing it
        assert!(book.ticks.contains_key(&21));

        // assert with total liq on tick once tracking is implemented
        // assert_eq!(book.ticks.get(&21).unwrap().quantity(), 300);

        // next ask tick should be updated to 21
        assert_eq!(book.next_ask_tick, 21);

        // We expect the USD balance to be equal to the quantity filled at each tick times the prices at each tick
        assert_eq!(order.owner().balance(Currency::USD), 300 * (10 + 13 + 14));
    }

    // implement test for run_market_bid, which is similar to ask but in the opposite tick direction
    #[test]
    fn test_run_market_bid() {
        let mut book = Orderbook::new(0);

        // set next bid tick on book to tick 10
        book.next_bid_tick = 21;

        // create orders on ticks 10, 13, 14, and 21 (latter is next bid tick)
        create_limit_orders(&mut book, &mut 10, 3, 100, &OrderDirection::Bid);
        create_limit_orders(&mut book, &mut 13, 3, 100, &OrderDirection::Bid);
        create_limit_orders(&mut book, &mut 14, 3, 100, &OrderDirection::Bid);
        create_limit_orders(&mut book, &mut 21, 3, 100, &OrderDirection::Bid);

        // run market bid for 1000 quantity
        // since market depth is 1200 (four ticks with 300 each), this should fill up to tick 21
        let mut order = Order::new(
            13,
            0,
            0,
            Account::new(0, AccountType::Individual),
            OrderType::Market,
            OrderDirection::Bid,
            1000,
        );
        book.run_market_bid(&mut order, u64::MIN, 1000);

        // ticks 10, 13, and 14 should all be emptied and removed from the book
        assert!(!book.ticks.contains_key(&13));
        assert!(!book.ticks.contains_key(&14));
        assert!(!book.ticks.contains_key(&21));

        // tick 10 should still be in the book and have 200 quantity left
        assert!(book.ticks.contains_key(&10));

        // assert with total liq on tick once tracking is implemented
        // assert_eq!(book.ticks.get(&10).unwrap().quantity(), 200);

        // next bid tick should be updated to 10
        assert_eq!(book.next_bid_tick, 10);

        // We expect the OSMO balance to be equal to the quantity filled on each tick
        assert_eq!(order.owner().balance(Currency::OSMO), 300 * 3 + 100 * 1);
    }

    // now write test with cutoff on 13
    #[test]
    fn test_run_market_bid_with_end_tick() {
        let mut book = Orderbook::new(0);

        // set next bid tick on book to tick 10
        book.next_bid_tick = 21;

        // create orders on ticks 10, 13, 14, and 21 (latter is next bid tick)
        create_limit_orders(&mut book, &mut 10, 3, 100, &OrderDirection::Bid);
        create_limit_orders(&mut book, &mut 13, 3, 100, &OrderDirection::Bid);
        create_limit_orders(&mut book, &mut 14, 3, 100, &OrderDirection::Bid);
        create_limit_orders(&mut book, &mut 21, 3, 100, &OrderDirection::Bid);

        // run market bid for 1000 quantity
        // since market depth is 1200 (four ticks with 300 each), this should fill up to tick 13
        let mut order = Order::new(
            13,
            0,
            0,
            Account::new(0, AccountType::Individual),
            OrderType::Market,
            OrderDirection::Bid,
            1000,
        );
        book.run_market_bid(&mut order, 13, 1000);

        // ticks 14 and 21 should be emptied and removed from the book
        assert!(!book.ticks.contains_key(&14));
        assert!(!book.ticks.contains_key(&21));

        // ticks 10 and 13 should still be untouched
        assert!(book.ticks.contains_key(&10));
        assert!(book.ticks.contains_key(&13));

        // assert with total liq on tick once tracking is implemented
        // assert_eq!(book.ticks.get(&10).unwrap().quantity(), 300);
        // assert_eq!(book.ticks.get(&13).unwrap().quantity(), 300);

        // next bid tick should be updated to 10
        assert_eq!(book.next_bid_tick, 13);

        // We expect the OSMO balance to be equal to the quantity filled on each tick
        assert_eq!(order.owner().balance(Currency::OSMO), 300 * 2);
    }
}