use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::{ValidAccountId, U128};

use super::level_table::LevelTable;
use super::order::{LimitOrder, BID, ASK};
use near_sdk::collections::{LookupMap};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, AccountId, PromiseOrValue,
    serde_json,

};

type Balances = LookupMap<AccountId, (u128, u128)>;
type Balance = LookupMap<AccountId, u128>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum FtOnTransferArgs {
    LinkMainnetAccount { account_id: ValidAccountId },
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct OrderBook {
    balances: Balances,

    token1_balance: Balance,
    token2_balance: Balance,

    token1_id: AccountId,
    token2_id: AccountId,

    bids: LevelTable,
    asks: LevelTable,

    max_bid: u128,
    min_bid: u128,
    max_ask: u128,
    min_ask: u128,

}

pub fn cmp(a: u128, b: u128, order: bool) -> bool {
    return match order {
        true => (a <= b),
        false => (a >= b)
    }
}

impl OrderBook {
    pub fn new(token1: AccountId, token2: AccountId) -> Self {
        OrderBook {
            balances: Balances::new(b"b".to_vec()),
            token1_balance: Balance::new(b"b".to_vec()),
            token2_balance: Balance::new(b"b".to_vec()),
            token1_id: token1,
            token2_id: token2,
            bids: LevelTable::new(BID),
            asks: LevelTable::new(ASK),
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

        level_range = match !order.side {
            BID => self.max_bid..=self.min_bid,
            ASK => self.min_ask..=self.max_ask, 
        };

        table = match !order.side {
            BID => &mut self.bids,
            ASK => &mut self.asks, 
        };
        
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
                    order.size -= level_order.size * price;
                    let out_order = table.get_level(price).pop().unwrap();

                    // pending_orders.insert(id, out_order);
                    out_order.execute_order();
                }
            }
        }

        if order.size > 0 {
            // add remaining to book
        }
    }

    pub fn async_fill(id) {
        pending_orders.get(id).fill();
    }

    pub fn internal_transfer(&mut self, from: AccountId, to: AccountId, value: u128, side: bool) {
        assert!(from != to);

        let token_balance = match side {
            BID => &mut self.token1_balance,
            ASK => &mut self.token2_balance, 
        };

            
        let new_from_balance = token_balance.get(&from).unwrap() - value;
        let new_to_balance = token_balance.get(&to).unwrap() + value;
            
        token_balance.insert(&from, &new_from_balance);
        token_balance.insert(&to, &new_to_balance);
    }

    pub fn internal_token_deposit(&mut self, sender: &AccountId, account_id: &AccountId, amount: u128) {
        let new_amount;

        if *account_id == self.token1_id {
            new_amount = self.token1_balance.get(&sender).unwrap_or(0) + amount;
            self.token1_balance.insert(&sender, &new_amount);
        } else {
            new_amount = self.token2_balance.get(&sender).unwrap_or(0) + amount;
            self.token2_balance.insert(&sender, &new_amount);
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
                (&self.token1_id == &token_account_id) || (&self.token2_id == &token_account_id)
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
