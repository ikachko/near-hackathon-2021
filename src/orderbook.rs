use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde_json::json;

use super::level_table::LevelTable;
use super::order::{LimitOrder, BID, ASK};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, AccountId, PromiseOrValue,
    serde_json, BorshStorageKey, Promise,
    PromiseIndex
};

use near_sdk::log;

// use std::Vector;

type BalanceMap = LookupMap<AccountId, u128>;
type PendingMap = LookupMap<u128, LimitOrder>;
type PromiseIdxVec = Vector<PromiseIndex>;


pub const A: bool = true;
pub const B: bool = false;
pub const EMPTY_PROMISE_CHAIN: u64 = 1_000_000_000;


#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum FtOnTransferArgs {
    LinkMainnetAccount { account_id: ValidAccountId },
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct OrderBook {
    tokenA_balance: BalanceMap,
    tokenB_balance: BalanceMap,

    tokenA_id: AccountId,
    tokenB_id: AccountId,

    bids: LevelTable,
    asks: LevelTable,

    pending: PendingMap,

    max_bid: u128,
    min_bid: u128,
    max_ask: u128,
    min_ask: u128,

}

impl Default for OrderBook {
    fn default() -> OrderBook {
        OrderBook {
            tokenA_balance: BalanceMap::new(b"ta".to_vec()),
            tokenB_balance: BalanceMap::new(b"tb".to_vec()),

            tokenA_id: AccountId::default(),
            tokenB_id: AccountId::default(),

            bids: LevelTable::default(),
            asks: LevelTable::default(),

            pending: PendingMap::new(b"s".to_vec()),

            max_bid: 0u128,
            min_bid: 0u128,
            max_ask: 0u128,
            min_ask: 0u128,
        }
    }
}

pub struct Trade {
    from: AccountId,
    to: AccountId,
    id_t: u128,
    it_m: u128,
    side: bool
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
   AccountBalance,
   LevelTableBid,
   LevelTableAsk,
   TokenABalance,
   TokenBBalance,
   Pending,
}


pub fn cmp(a: u128, b: u128, order: bool) -> bool {
    return match order {
        true => (a <= b),
        false => (a >= b)
    }
}

impl OrderBook {
    pub fn new(tokenA: AccountId, tokenB: AccountId) -> Self {
        OrderBook {
            tokenA_balance: BalanceMap::new(StorageKey::TokenABalance),
            tokenB_balance: BalanceMap::new(StorageKey::TokenBBalance),
            tokenA_id: tokenA,
            tokenB_id: tokenB,
            bids: LevelTable::new(BID),
            asks: LevelTable::new(ASK),
            pending: PendingMap::new(StorageKey::Pending),
            max_bid: 0,
            min_bid: 0,
            max_ask: 0,
            min_ask: 0,
        }
    }

    pub fn send_order(&mut self, o: LimitOrder) {
        self.try_match(o); // Return status
    }

    pub fn try_match(
        &mut self,
        mut order: LimitOrder,
    ) {
        let level_range;
        let table;
        let mut o = order.clone();
        let mut o_size = o.size;

        let mut promises = PromiseIdxVec::new(b"promises".to_vec());

        // let mut order_promise = Promise::new(env::current_account_id());
        let mut order_promise = EMPTY_PROMISE_CHAIN;

        order.lock();

        level_range = match !order.side {
            BID => self.max_bid..=self.min_bid,
            ASK => self.min_ask..=self.max_ask,
        };

        table = match !order.side {
            BID => &mut self.bids,
            ASK => &mut self.asks,
        };

        self.pending.insert(&o.id, &o);

        for price in level_range {
            if order.size == 0 {
                return;
            }

            if cmp(order.price, price, order.side) && !table.get_level(price).empty() {
                let mut level_order = table.get_level(price).get(0);

                if level_order.size > order.size * price {
                    level_order.sub(order.size * price);
                    order.size = 0;
                } else {
                    let matched_size = level_order.size * price;

                    order.size -= matched_size;

                    let (mut order_m, promise) = table.get_level(price).pop(&order.id);

                    order_promise = match order_promise {
                        EMPTY_PROMISE_CHAIN => promise.unwrap(),
                        _ => env::promise_and(&[order_promise, promise.unwrap()])
                    };

                    order_m.lock();

                    self.pending.insert(
                        &order_m.id,
                        &order_m
                    );
                }
            }
        }

        if order_promise != EMPTY_PROMISE_CHAIN {
            env::promise_then(
                order_promise,
                env::current_account_id(),
                b"order_finalization",
                json!({"id_t": order.id}).to_string().as_bytes(),
                0,
                5_000_000_000_000
            );
        }
    }

    pub fn order_finalization(&mut self, id_t: &u128) {
        assert!(env::current_account_id() == env::predecessor_account_id());

        let taker_order = self.pending.get(&id_t).unwrap();

        if taker_order.size > 0 {
            log!("Order {} was filled partially. Remaining size {}", taker_order.id, taker_order.size);
        }
    }

    pub fn on_execute(&mut self, id_t: &u128, id_m: &u128, status: bool) {
        let mut order_t = self.pending.get(&id_t).unwrap();
        let order_m = self.pending.get(&id_m).unwrap();

        if status {
            let match_size = order_m.price * order_m.size;
            // transfer tokenA from taker to maker
            match order_t.side {
                BID => {
                    // Taker gives A to Maker
                    self.internal_transfer(
                        &order_t.address,
                        &order_m.address,
                        &order_m.size,
                        A
                    );
                    // Maker gives B to Taker
                    self.internal_transfer(
                        &order_m.address,
                        &order_t.address,
                        &match_size,
                        B
                    );
                },
                ASK => {
                    // Taker gives B to Maker
                    self.internal_transfer(
                        &order_t.address,
                        &order_m.address,
                        &order_m.size,
                        A
                    );
                    // Maker gives A to Taker
                    self.internal_transfer(
                        &order_m.address,
                        &order_t.address,
                        &match_size,
                        B
                    );
                }
            }
            order_t.size -= match_size;
        }
        self.pending.remove(&id_m);
    }

    pub fn internal_transfer(
        &mut self,
        from: &AccountId,
        to: &AccountId,
        value: &u128,
        token: bool
    ) {
        assert!(from != to);

        let token_balance = match token {
            A => &mut self.tokenA_balance,
            B => &mut self.tokenB_balance,
        };


        let new_from_balance = token_balance.get(&from).unwrap() - value;
        let new_to_balance = token_balance.get(&to).unwrap() + value;

        token_balance.insert(&from, &new_from_balance);
        token_balance.insert(&to, &new_to_balance);
    }

    pub fn internal_token_deposit(&mut self, sender: &AccountId, account_id: &AccountId, amount: u128) {
        let new_amount;

        if *account_id == self.tokenA_id {
            new_amount = self.tokenA_balance.get(&sender).unwrap_or(0) + amount;
            self.tokenA_balance.insert(&sender, &new_amount);
        } else {
            new_amount = self.tokenB_balance.get(&sender).unwrap_or(0) + amount;
            self.tokenB_balance.insert(&sender, &new_amount);
        }
    }

    pub fn push_order(&mut self, o: LimitOrder) {
        match o.side {
            BID => self.bids.add_order(o),
            ASK => self.asks.add_order(o),
        }
    }
}

const ERR_FAILED_TO_PARSE_FT_ON_TRANSFER_MSG: &str = "ERR_FAILED_TO_PARSE_FT_ON_TRANSFER_MSG";
const ERR_INVALID_FT_ACCOUNT_ID: &str = "ERR_INVALID_FT_ACCOUNT_ID";

#[near_bindgen]
impl FungibleTokenReceiver for OrderBook {
    #[allow(unused_variables)]
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let args: FtOnTransferArgs =
            serde_json::from_str(&msg).expect(ERR_FAILED_TO_PARSE_FT_ON_TRANSFER_MSG);
        let token_account_id = env::predecessor_account_id();

        assert!(
            (
                (&self.tokenA_id == &token_account_id) || (&self.tokenB_id == &token_account_id)
            ),
            "{}",
            ERR_INVALID_FT_ACCOUNT_ID
        );

        match args {
            FtOnTransferArgs::LinkMainnetAccount { account_id } => {
                self.internal_token_deposit(sender_id.as_ref(), account_id.as_ref(), amount.0);
            }
        }
        PromiseOrValue::Value(0.into())
    }
}
