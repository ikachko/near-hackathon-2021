use super::order::{LimitOrder};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::{env, near_bindgen};

type Orders = Vector<LimitOrder>;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Level {
    pub volume: u128,
    pub orders: Orders
}

impl Level {
    pub fn new() -> Self {
        Level {
            volume: 0,
            orders: Orders::new(0)
        }
    }

    pub fn get(&mut self, idx: u64) -> LimitOrder {
        self.orders.get(idx).unwrap()
    }

    pub fn empty(self) -> bool {
        self.orders.is_empty()
    }

    pub fn push(&mut self, order: LimitOrder) {
        self.volume += order.size;
        self.orders.push(&order)
    }

    pub fn pop(&mut self) -> Option<LimitOrder>{
        let order = self.orders.pop();
        self.volume -= order.as_ref().unwrap().size;
        order
    }
}
