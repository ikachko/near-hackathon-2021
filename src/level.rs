use super::order::{LimitOrder};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::{env, near_bindgen, Promise};

type OrdersVec = Vector<LimitOrder>;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Level {
    pub volume: u128,
    pub orders: OrdersVec
}

impl Level {
    pub fn new() -> Self {
        Level {
            volume: 0,
            orders: OrdersVec::new(env::random_seed())
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

    // pub fn pop(&mut self, id_t: &u128) -> (LimitOrder, Option<Promise>) {
    pub fn pop(&mut self, id_t: &u128) -> (LimitOrder, Option<u64>) {
        let mut order = self.orders.pop().unwrap();
        let mut order_copy = order.clone();
        self.volume -= order.size;

        if order.callable != "" {
            return (order, Some(order_copy.execute_order(id_t)))
        }

        (order, None)
    }
}
