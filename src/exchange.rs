use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use orderbook::OrderBook;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct KKSwap {
    orderbook: OrderBook
}

// #[near_bindgen]
// impl KKSwap {
//     #[init]
//     pub fn new() -> Self {
//         self.orderbook = OrderBook();
        
//         Self
//     }

//     pub fn send_order(input_vec &[&str]) {

//     }
// }