use getset::Getters;
use crate::bank::account::Account;
use crate::bank::currency::Currency;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum OrderDirection {
    Bid,
    Ask,
}

#[derive(Getters, Clone, Debug)]
pub struct Order {
    #[get = "pub"]
    order_id: u64,
    #[get = "pub"]
    tick_id: u64,
    #[get = "pub"]
    book_id: u64,
    #[get = "pub"]
    owner: Account,
    #[get = "pub"]
    order_type: OrderType,
    #[get = "pub"]
    order_direction: OrderDirection,
    #[get = "pub"]
    quantity: u64,
}

impl Order {
    pub fn new(
        order_id: u64,
        tick_id: u64,
        book_id: u64,
        owner: Account,
        order_type: OrderType,
        order_direction: OrderDirection,
        quantity: u64,
    ) -> Order {
        Order {
            order_id: order_id,
            tick_id: tick_id,
            book_id: book_id,
            owner: owner,
            order_type: order_type,
            order_direction: order_direction,
            quantity: quantity,
        }
    }

    // implement public function for filling an order. Should return the remaining amount of the input quantity.
    pub fn fill_order(&mut self, fill_quantity: u64) -> u64 {
        let mut remaining_quantity = fill_quantity;
        if self.quantity > fill_quantity {
            self.quantity -= fill_quantity;
            remaining_quantity = 0;
        } else {
            remaining_quantity = fill_quantity - self.quantity;
            self.quantity = 0;
        }

        // If order was a bid, this means osmo was bought, so we need to update owner's balance with just quantity osmo.
        // If it was an ask, this means osmo was sold, so we need to update owner's balance with just quantity * price (tick).
        let amount_filled = fill_quantity - remaining_quantity;
        let price_per_sold_unit = self.tick_id;
        self.distribute_filled_assets(amount_filled, price_per_sold_unit);

        return remaining_quantity;
    }

    pub fn set_quantity(&mut self, quantity: u64) {
        self.quantity = quantity;
    }

    // Send order owner the appropriate amount of filled assets depending on their original order.
    pub fn distribute_filled_assets(&mut self, amount_filled: u64, price_per_filled_unit: u64) {
        match self.order_direction {
            OrderDirection::Bid => {
                self.owner.deposit(Currency::OSMO, amount_filled);
            },
            OrderDirection::Ask => {
                self.owner.deposit(Currency::USD, amount_filled * price_per_filled_unit);
            },
        }
    }
}

// write unit tests for fill_order
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bank::account::AccountType;

    #[test]
    fn test_fill_order_bid() {
        let mut order = Order::new(
            0,
            5,
            0,
            Account::new(0, AccountType::Individual),
            OrderType::Limit,
            OrderDirection::Bid,
            100,
        );

        // All of the input is consumed, so the remaining input quantity should be 0.
        assert_eq!(order.fill_order(50), 0);

        // There is still 50 left to fill, so the order quantity should be 50.
        assert_eq!(*order.quantity(), 50);

        // Fill another 50 units.
        assert_eq!(order.fill_order(50), 0);

        // There is no more quantity left to fill, so the order quantity should be 0.
        assert_eq!(*order.quantity(), 0);

        // Attempting to fill another 50 should be unsuccessful and just return the full input amount.
        assert_eq!(order.fill_order(50), 50);

        // Sanity check that the order quantity is still 0.
        assert_eq!(*order.quantity(), 0);

        // Sanity check that the owner's balance is 100.
        assert_eq!(order.owner.balance(Currency::OSMO), 100);
    }

    // implement test for ask
    #[test]
    fn test_fill_order_ask() {
        let mut order = Order::new(
            0,
            5,
            0,
            Account::new(0, AccountType::Individual),
            OrderType::Limit,
            OrderDirection::Ask,
            100,
        );

        // All of the input is consumed, so the remaining input quantity should be 0.
        assert_eq!(order.fill_order(50), 0);

        // There is still 50 left to fill, so the order quantity should be 50.
        assert_eq!(*order.quantity(), 50);

        // Fill another 50 units.
        assert_eq!(order.fill_order(50), 0);

        // There is no more quantity left to fill, so the order quantity should be 0.
        assert_eq!(*order.quantity(), 0);

        // Attempting to fill another 50 should be unsuccessful and just return the full input amount.
        assert_eq!(order.fill_order(50), 50);

        // Sanity check that the order quantity is still 0.
        assert_eq!(*order.quantity(), 0);

        // Sanity check that the owner's balance is 500 USD since 100 OSMO was sold at tick 5 (5 USD per OSMO).
        assert_eq!(order.owner.balance(Currency::USD), 500);
    }
}
