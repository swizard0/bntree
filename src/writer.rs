pub mod sketch {
    use std::cmp::min;

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Level {
        pub index: usize,
        pub blocks_count: usize,
        pub items_count: usize,
    }

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
}

pub mod plan {
    use super::sketch;

    pub fn build<'s>(sketch: &'s sketch::Tree) -> Plan<'s> {
        Plan {
            sketch,
            cursors: sketch
                .levels()
                .iter()
                .rev()
                .map(|level| LevelCursor {
                    level,
                    block_index: 0,
                    block_cursor: BlockCursor::Start,
                    items_remain: level.items_count,
                })
                .collect(),
            level_curr: 0,
            level_base: 0,
        }
    }

    pub struct Plan<'s> {
        sketch: &'s sketch::Tree,
        cursors: Vec<LevelCursor<'s>>,
        level_curr: usize,
        level_base: usize,
    }

    struct LevelCursor<'s> {
        level: &'s sketch::Level,
        block_index: usize,
        block_cursor: BlockCursor,
        items_remain: usize,
    }

    enum BlockCursor {
        Start,
        Write { index: usize, },
    }

    impl<'s> Iterator for Plan<'s> {
        type Item = Instruction<'s>;

        fn next(&mut self) -> Option<Self::Item> {
            while self.level_curr < self.cursors.len() {
                let cursor = &mut self.cursors[self.level_curr];
                if cursor.items_remain == 0 {
                    let instruction = Instruction {
                        level: cursor.level,
                        block_index: cursor.block_index,
                        op: Op::BlockFinish,
                    };
                    self.level_base += 1;
                    self.level_curr = self.level_base;
                    return Some(instruction);
                }
                match cursor.block_cursor {
                    BlockCursor::Start => {
                        cursor.block_cursor = BlockCursor::Write { index: 0, };
                        return Some(Instruction {
                            level: cursor.level,
                            block_index: cursor.block_index,
                            op: Op::BlockStart,
                        });
                    },
                    BlockCursor::Write { index, } if index < self.sketch.block_size() => {
                        cursor.block_cursor = BlockCursor::Write { index: index + 1, };
                        cursor.items_remain -= 1;
                        self.level_curr = self.level_base;
                        return Some(Instruction {
                            level: cursor.level,
                            block_index: cursor.block_index,
                            op: Op::WriteItem { block_item_index: index, },
                        });
                    },
                    BlockCursor::Write { .. } => {
                        cursor.block_cursor = BlockCursor::Start;
                        let block_index = cursor.block_index;
                        cursor.block_index += 1;
                        self.level_curr += 1;
                        return Some(Instruction {
                            level: cursor.level,
                            block_index,
                            op: Op::BlockFinish,
                        });
                    },
                }
            }

            None
        }
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Instruction<'s> {
        pub level: &'s sketch::Level,
        pub block_index: usize,
        pub op: Op,
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Op {
        BlockStart,
        WriteItem { block_item_index: usize, },
        BlockFinish,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        sketch,
        plan,
    };

    #[test]
    fn tree17_4() {
        let sketch = sketch::Tree::new(17, 4);
        assert_eq!(
            sketch.levels(),
            &[
                sketch::Level { index: 0, blocks_count: 1, items_count: 4 },
                sketch::Level { index: 1, blocks_count: 4, items_count: 13 },
            ]
        );

        let script: Vec<_> = plan::build(&sketch).collect();
        assert_eq!(
            script,
            vec![
                plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::BlockStart },
                plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 0 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 1 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 2 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 3 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::BlockFinish },
                plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::BlockStart },
                plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 0 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::BlockStart },
                plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 0 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 1 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 2 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 3 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::BlockFinish },
                plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 1 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::BlockStart },
                plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 0 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 1 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 2 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 3 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::BlockFinish },
                plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 2 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 3, op: plan::Op::BlockStart },
                plan::Instruction { level: &sketch.levels()[1], block_index: 3, op: plan::Op::WriteItem { block_item_index: 0 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 3, op: plan::Op::BlockFinish },
                plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 3 } },
                plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::BlockFinish },
            ],
        );
    }

    #[test]
    fn tree17_3() {
        let sketch = sketch::Tree::new(17, 3);
        assert_eq!(
            sketch.levels(),
            &[
                sketch::Level { index: 0, blocks_count: 1, items_count: 3 },
                sketch::Level { index: 1, blocks_count: 3, items_count: 9 },
                sketch::Level { index: 2, blocks_count: 2, items_count: 5 },
            ]
        );

        let script: Vec<_> = plan::build(&sketch).collect();
        assert_eq!(
            script,
            vec![
                plan::Instruction { level: &sketch.levels()[2], block_index: 0, op: plan::Op::BlockStart },
                plan::Instruction { level: &sketch.levels()[2], block_index: 0, op: plan::Op::WriteItem { block_item_index: 0 } },
                plan::Instruction { level: &sketch.levels()[2], block_index: 0, op: plan::Op::WriteItem { block_item_index: 1 } },
                plan::Instruction { level: &sketch.levels()[2], block_index: 0, op: plan::Op::WriteItem { block_item_index: 2 } },
                plan::Instruction { level: &sketch.levels()[2], block_index: 0, op: plan::Op::BlockFinish },
                plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::BlockStart },
                plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 0 } },
                plan::Instruction { level: &sketch.levels()[2], block_index: 1, op: plan::Op::BlockStart },
                plan::Instruction { level: &sketch.levels()[2], block_index: 1, op: plan::Op::WriteItem { block_item_index: 0 } },
                plan::Instruction { level: &sketch.levels()[2], block_index: 1, op: plan::Op::WriteItem { block_item_index: 1 } },
                plan::Instruction { level: &sketch.levels()[2], block_index: 1, op: plan::Op::BlockFinish },
                plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 1 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 2 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::BlockFinish },
                plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::BlockStart },
                plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 0 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::BlockStart },
                plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 0 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 1 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 2 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::BlockFinish },
                plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 1 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::BlockStart },
                plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 0 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 1 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 2 } },
                plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::BlockFinish },
                plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 2 } },
                plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::BlockFinish },
            ],
        );
    }
}

// #[derive(Debug)]
// pub enum BuildError<IE, WE> {
//     Iter(IE),
//     Writer(WE),
// }

// pub trait Writer {
//     type Item;
//     type Error;
//     type Position;
//     type Block;
//     type Result;

//     fn empty_pos(&self) -> Self::Position;
//     fn make_block(&self) -> Result<Self::Block, Self::Error>;
//     fn write_item(&mut self, block: &mut Self::Block, item: Self::Item, child_block_pos: Self::Position) -> Result<(), Self::Error>;
//     fn flush_block(&mut self, block: Self::Block) -> Result<Self::Position, Self::Error>;
//     fn finish(self, root_block_pos: Self::Position) -> Result<Self::Result, Self::Error>;
// }

// pub fn build<I, W, E>(
//     mut src: I,
//     src_len: usize,
//     mut dst: W,
//     min_tree_height: usize,
//     max_block_size: usize,
// ) ->
//     Result<W::Result, BuildError<E, W::Error>>
// where
//     I: Iterator<Item = Result<W::Item, E>>,
//     W: Writer,
// {
//     let mut index_block_size = max_block_size + 1;
//     let mut tree_height = min_tree_height;
//     while src_len > 0 && index_block_size > max_block_size {
//         index_block_size = (src_len as f64).powf(1.0 / tree_height as f64) as usize;
//         tree_height += 1;
//     }
//     let root_block_pos = build_block(&mut src, 0, src_len, index_block_size, &mut dst)?;
//     dst.finish(root_block_pos).map_err(|e| BuildError::Writer(e))
// }

// fn build_block<T, E, P, I, IE, W>(
//     src: &mut I,
//     block_start: usize,
//     block_end: usize,
//     block_size: usize,
//     dst: &mut W,
// )
//     -> Result<P, BuildError<IE, E>>
// where I: Iterator<Item = Result<T, IE>>,
//       W: Writer<Item = T, Error = E, Position = P>,
// {
//     let interval = block_end - block_start;
//     let (index_start, index_inc) = if interval > block_size {
//         (interval - (interval / block_size * (block_size - 1)) - 1, interval / block_size)
//     } else {
//         (0, 1)
//     };

//     let mut block = dst.make_block()
//         .map_err(|e| BuildError::Writer(e))?;
//     let mut node_block_start = block_start;
//     let mut node_block_end = block_start + index_start;
//     while node_block_end < block_end {
//         let child_block_pos = if node_block_start < node_block_end {
//             build_block(src, node_block_start, node_block_end, block_size, dst)?
//         } else {
//             dst.empty_pos()
//         };

//         if let Some(maybe_item) = src.next() {
//             dst.write_item(&mut block, maybe_item.map_err(|e| BuildError::Iter(e))?, child_block_pos)
//                 .map_err(|e| BuildError::Writer(e))?;
//         } else {
//             break
//         }

//         node_block_start = node_block_end + 1;
//         node_block_end += index_inc;
//     }

//     dst.flush_block(block).map_err(|e| BuildError::Writer(e))
// }
