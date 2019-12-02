use serde_derive::{
    Serialize,
    Deserialize,
};

use std::cmp::min;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Level {
    pub index: usize,
    pub blocks_count: usize,
    pub items_count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Tree {
    levels: Vec<Level>,
    block_size: usize,
    items_total: usize,
}

impl Tree {
    pub fn new(items_total: usize, block_size: usize) -> Tree {
        let mut blocks_count = (items_total as f64 / block_size as f64).ceil() as usize;
        let mut levels = Vec::new();
        let mut items_remain = items_total;
        for layer in 0 .. {
            if items_remain == 0 {
                break;
            }
            let layer_max_blocks = (block_size as f64).powi(layer as i32) as usize;
            let layer_blocks = min(layer_max_blocks, blocks_count);
            let layer_max_items = layer_blocks * block_size;
            let layer_items = min(layer_max_items, items_remain);
            levels.push(Level {
                index: layer,
                blocks_count: layer_blocks,
                items_count: layer_items,
            });
            blocks_count -= layer_blocks;
            items_remain -= layer_items;
        }
        Tree { levels, block_size, items_total, }
    }

    pub fn levels(&self) -> &[Level] {
        &self.levels
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn items_total(&self) -> usize {
        self.items_total
    }
}
