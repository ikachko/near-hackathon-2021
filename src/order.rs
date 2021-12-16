use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, ext_contract};
use near_sdk::env::{promise_return, promise_then, current_account_id};
use near_sdk::AccountId;
use near_sdk::env::signer_account_id;
use std::option::Option;

pub const BID: bool = true;
pub const ASK: bool = false;

#[ext_contract(ext_callable)]
trait Callable {
    fn execute() -> bool;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LimitOrder {
    pub timestamp: i64,
    pub address: AccountId,
    pub callable: AccountId,
    // pub id: u32,
    pub side: bool,
    pub price: u128,
    pub size: u128,
    pub status: bool,
    pub pending: bool,
}

#[near_bindgen]
impl LimitOrder {
    pub fn new(
        t: i64,
        c: AccountId,
        sd: bool,
        p: u128,
        sz: u128,
    ) -> Self {

        LimitOrder {
            timestamp: t,
            address: signer_account_id(),
            callable: c,
            side: sd,
            price: p,
            size: sz,
            status: false,
            pending: false,
        }
    }

    pub fn execute_order(&mut self) -> bool {
        let promise = ext_callable::execute(&self.callable, 0, 5_000_000_000_000);

        let promise1 = promise_then(
            promise,
            current_account_id(),
            "clear()",

        );

        promise_return(promise)
    }

    pub fn on_execute(promise_result: bool) {

    }

    pub fn sub(&mut self, a: u128) {
        self.size -= a;
    }
}
