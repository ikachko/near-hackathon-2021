use super::order::LimitOrder;
use super::level::Level;
use near_sdk::collections::UnorderedMap;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen};


type LevelsMap = UnorderedMap<u128, Level>;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LevelTable {
    pub table: LevelsMap,
    table_side: bool
}

// #[near_bindgen]
impl LevelTable {
    pub fn new(side: bool) -> Self {
        LevelTable {
            table: LevelsMap::new(env::random_seed()),
            table_side: side, 
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
        }
    }

    pub fn get_level(&mut self, idx: u128) -> Level {
        self.table.get(&idx).unwrap()
    }
}
