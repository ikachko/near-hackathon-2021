use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, ext_contract, AccountId, Promise};
use near_sdk::env::{keccak256, signer_account_id};
use near_sdk::env;
use near_sdk::serde_json::{self, json};

use serde::Serialize;

pub const BID: bool = true;
pub const ASK: bool = false;

#[ext_contract(ext_callable)]
trait Callable {
    fn execute(id_m: u128, id_t: u128);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize)]
pub struct LimitOrder {
    pub timestamp: i128,
    pub address: String,
    pub callable: String,
    pub id: u128,
    pub side: bool,
    pub price: u128,
    pub size: u128,
    pub status: bool,
    pub pending: bool,
}

impl Default for LimitOrder {
    fn default() -> LimitOrder {
        LimitOrder {
            timestamp: 0i128,
            address: String::default(),
            callable: String::default(),
            id: 0u128,
            side: bool::default(),
            price: 0u128,
            size: 0u128,
            status: bool::default(),
            pending: bool::default(),
        }
    }
}

#[near_bindgen]
impl LimitOrder {
    pub fn new(
        t: i128,
        c: AccountId,
        sd: bool,
        p: u128,
        sz: u128,
    ) -> Self {
        let c_copy = c.clone();
        let hash = keccak256((
            c.clone()
            + &t.to_string()
            + &p.to_string()
            + &sz.to_string()
            + &sd.to_string()
        ).as_bytes());

        LimitOrder {
            timestamp: t,
            address: signer_account_id(),
            id: u128::from_be_bytes(hash[..16].try_into().unwrap()),
            callable: c_copy,
            side: sd,
            price: p,
            size: sz,
            status: false,
            pending: false,
        }
    }

    pub fn execute_order(&mut self, id_t: &u128) -> Promise {

        // env::promise_create(
        //     self.callable.clone(),
        //     b"execute",
        //     json!({"id_m": self.id, "id_t": *id_t}).to_string().as_bytes(),
        //     0,
        //     5_000_000_000_000
        // )
        ext_callable::execute(
            self.id,
            *id_t,
            &self.callable,
            0,
            5_000_000_000_000
        )
    }

    pub fn lock(&mut self) {
        self.pending = true;
    }

    pub fn sub(&mut self, a: u128) {
        self.size -= a;
    }
}
