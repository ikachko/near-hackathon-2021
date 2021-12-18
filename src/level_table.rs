use super::order::LimitOrder;
use super::level::Level;
use super::order::{BID, ASK};

use near_sdk::collections::UnorderedMap;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen};

type LevelsMap = UnorderedMap<u128, Level>;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LevelTable {
    pub table: LevelsMap,
    table_side: bool,
    min_level: u128,
    max_level: u128,
}

impl Default for LevelTable {
    fn default() -> LevelTable {
        LevelTable {
            table: LevelsMap::new(b"t".to_vec()),
            table_side: bool::default(),
            min_level: 0u128,
            max_level: 0u128
        }
    }
}

impl LevelTable {
    pub fn new(side: bool) -> Self {
        LevelTable {
            table: LevelsMap::new(env::random_seed()),
            table_side: side,
            min_level: 80u128,
            max_level: 100u128
        }
    }

    pub fn add_order(&mut self, o: LimitOrder) {
        if o.side != self.table_side {
            return;
        }

        if !self.table.get(&o.price).unwrap().empty() {
            self.table.get(&o.price).unwrap().push(o);
        } else {
            let mut new_level = Level::new();
            let price = o.price;

            new_level.push(o);

            self.table.insert(
                &price,
                &new_level
            );

            if price > self.max_level {
                self.max_level = price
            }
            if price < self.min_level {
                self.min_level = price
            }
        }
    }

    pub fn get_min_level(&mut self) -> u128 {
        self.min_level
    }

    pub fn get_max_level(&mut self) -> u128 {
        self.max_level
    }

    pub fn get_level(&mut self, idx: u128) -> Level {
        self.table.get(&idx).unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};
    use near_sdk::test_utils::{VMContextBuilder};
    // use near_sdk::env::{Va}


    fn get_context(predecessor_account_id: String, storage_usage: u64) -> VMContext {
        VMContext {
            current_account_id: "kkdex.testnet".to_string(),
            signer_account_id: "kkdex.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn initialize() {
        
    }
}