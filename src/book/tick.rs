use super::order::{Order, OrderType};
use getset::Getters;
use std::collections::VecDeque;
use crate::bank::account::{Account, AccountType};

#[derive(Getters, Debug)]
pub struct Tick {
    #[get = "pub"]
    tick_id: u64,
    #[get = "pub"]
    next_order: u64,
    #[get = "pub"]
    orders: VecDeque<Order>,
    #[get = "pub"]
    total_orders: u64,
}

// implement public constructor and getters for all fields
impl Tick {
    pub fn new(tick_id: u64) -> Tick {
        Tick {
            tick_id: tick_id,
            next_order: 0,
            orders: VecDeque::new(),
            total_orders: 0,
        }
    }

    // fill_tick fills as much of the tick as possible with the given quantity.
    // It returns the remaining portion of the input quantity (0 if the whole input is consumed).
    pub fn fill_tick(&mut self, quantity: u64) -> u64 {
        let mut remaining_quantity = quantity;

        while remaining_quantity > 0 && self.orders.len() > 0 {
            let order = self.orders.front_mut().unwrap();
            remaining_quantity = order.fill_order(remaining_quantity);
            if order.quantity() == &0 {
                self.orders.pop_front();
            }
        }
        remaining_quantity
    }

    // Places limit order on tick
    pub fn place_limit(&mut self, order: Order) -> Result<(), &'static str> {
        if order.order_type() != &OrderType::Limit {
            return Err("Order is not a limit order");
        }
        self.orders.push_back(order);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::book::order::{OrderDirection, OrderType};

    // Helper function for placing orders on a tick (manual placement to avoid testing co-dependency)
    fn place_orders(tick: &mut Tick, num_orders: u64, quantity_per_order: u64) {
        for i in 0..num_orders {
            let order = Order::new(
                i,
                *tick.tick_id(),
                0,
                Account::new(i, AccountType::Individual),
                OrderType::Market,
                OrderDirection::Bid,
                quantity_per_order,
            );
            tick.orders.push_back(order);
        }
    }

    #[test]
    fn test_fill_tick() {
        // Place 10 orders of 10 quantity each on tick 0
        let mut tick = Tick::new(0);
        place_orders(&mut tick, 10, 10);

        // Fill 50 quantity on the tick. Since the whole tick is filled, the remaining quantity should be 0.
        assert_eq!(tick.fill_tick(55), 0);

        // Check that the five filled orders were removed from the tick.
        // The partially filled order should still be there.
        assert_eq!(tick.orders.len(), 5);

        // Fill the remaining 50 quantity on the tick.
        // Since there is only 45 quantity left on the tick, this fills the whole tick and overflows 5 units.
        assert_eq!(tick.fill_tick(50), 5);

        // The tick should have zero orders remaining.
        assert_eq!(tick.orders.len(), 0);
    }

    #[test]
    fn test_place_limit() {
        let mut tick = Tick::new(0);
        let order = Order::new(
            0,
            *tick.tick_id(),
            0,
            Account::new(0, AccountType::Individual),
            OrderType::Limit,
            OrderDirection::Bid,
            100,
        );

        // Place limit on tick
        let result = tick.place_limit(order);

        // Check that result returned was not an error
        assert_eq!(result.is_err(), false);

        // Check that the tick's queue was correctly updated
        assert_eq!(tick.orders.len(), 1);
    }

    #[test]
    fn test_place_limit_error() {
        let mut tick = Tick::new(0);

        // Attempt to place market order
        let order = Order::new(
            0,
            *tick.tick_id(),
            0,
            Account::new(0, AccountType::Individual),
            OrderType::Market,
            OrderDirection::Bid,
            100,
        );

        // Assert that correct error is returned
        let result = tick.place_limit(order);
        assert_eq!(result.unwrap_err(), "Order is not a limit order");

        // Assert that tick's queue was not updated
        assert_eq!(tick.orders.len(), 0);
    }
}
