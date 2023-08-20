use std::collections::HashMap;
use super::currency::Currency;

// enum for AccountType between individual and orderbook
#[derive(Clone, Debug)]
pub enum AccountType {
    Individual,
    Orderbook,
}

#[derive(Clone, Debug)]
pub struct Account {
    account_id: u64,
    balances: HashMap<Currency, u64>,
    account_type: AccountType,
}

impl Account {
    pub fn new(acc_id: u64, acc_type: AccountType) -> Account {
        Account {
            account_id: acc_id,
            balances: HashMap::new(),
            account_type: acc_type,
        }
    }

    pub fn deposit(&mut self, currency: Currency, amount: u64) {
        let balance = self.balances.entry(currency).or_insert(0);
        *balance += amount;
    }

    // withdraw but return Result error if insufficient funds
    pub fn withdraw(&mut self, currency: Currency, amount: u64) -> Result<(), &'static str> {
        let balance = self.balances.entry(currency).or_insert(0);
        if *balance < amount {
            return Err("Insufficient funds");
        }
        *balance -= amount;
        Ok(())
    }

    // check balance
    pub fn balance(&self, currency: Currency) -> u64 {
        *self.balances.get(&currency).unwrap_or(&0)
    }
}
