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
        pub sketch: &'s sketch::Tree,
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

pub mod fold {
    use super::{sketch, plan};

    pub fn fold_levels<'s, B, S>(sketch: &'s sketch::Tree) -> FoldLevels<'s, B, S> {
        FoldLevels {
            plan: plan::build(sketch),
            levels: sketch
                .levels()
                .iter()
                .map(|level| Level { level, state: Some(LevelState::Init), })
                .collect(),
        }
    }

    pub struct FoldLevels<'s, B, S> {
        plan: plan::Plan<'s>,
        levels: Vec<Level<'s, B, S>>,
    }

    struct Level<'s, B, S> {
        level: &'s sketch::Level,
        state: Option<LevelState<B, S>>,
    }

    enum LevelState<B, S> {
        Init,
        Active {
            level_seed: S,
            block: B,
            block_index: usize,
        },
        Flushed {
            level_seed: S,
        },
    }

    impl<'s, B, S> FoldLevels<'s, B, S> {
        pub fn next(mut self) -> Instruction<'s, B, S> {
            match self.plan.next() {
                None =>
                    Instruction::Done(Done { fold_levels: self, }),
                Some(plan::Instruction { level, block_index, op: plan::Op::BlockStart, }) => {
                    assert!(level.index < self.levels.len());
                    match self.levels[level.index].state.take() {
                        None =>
                            panic!("level state left in invalid state for BlockStart"),
                        Some(LevelState::Init) => {
                            assert_eq!(block_index, 0);
                            Instruction::VisitLevel(VisitLevel {
                                level,
                                next: VisitLevelNext {
                                    fold_levels: self,
                                    level,
                                },
                            })
                        },
                        Some(LevelState::Active { block_index: prev_block_index, .. }) =>
                            panic!("new block {} while previous block {} is not finished on level {} for BlockStart",
                                   block_index, prev_block_index, level.index),
                        Some(LevelState::Flushed { level_seed, }) => {
                            assert!(block_index > 0);
                            Instruction::VisitBlockStart(VisitBlockStart {
                                level,
                                level_seed,
                                block_index,
                                next: VisitBlockStartNext {
                                    fold_levels: self,
                                    level,
                                    block_index,
                                },
                            })
                        },
                    }
                },
                Some(plan::Instruction { level, block_index, op: plan::Op::WriteItem { block_item_index, }, .. }) => {
                    assert!(level.index < self.levels.len());
                    match self.levels[level.index].state.take() {
                        None =>
                            panic!("level state left in invalid state for WriteItem"),
                        Some(LevelState::Init) =>
                            panic!("level and block are not initialized for WriteItem"),
                        Some(LevelState::Active { level_seed, block, block_index: active_block_index, }) => {
                            assert!(active_block_index == block_index);
                            Instruction::VisitItem(VisitItem {
                                level,
                                level_seed,
                                block,
                                block_index,
                                block_item_index,
                                next: VisitItemNext {
                                    fold_levels: self,
                                    level,
                                    block_index,
                                },
                            })
                        },
                        Some(LevelState::Flushed { .. }) =>
                            panic!("block is already flushed for WriteItem"),
                    }
                },
                Some(plan::Instruction { level, block_index, op: plan::Op::BlockFinish, }) => {
                    assert!(level.index < self.levels.len());
                    match self.levels[level.index].state.take() {
                        None =>
                            panic!("level state left in invalid state for BlockFinish"),
                        Some(LevelState::Init) =>
                            panic!("level and block are not initialized for BlockFinish"),
                        Some(LevelState::Active { level_seed, block, block_index: active_block_index, }) => {
                            assert!(active_block_index == block_index);
                            Instruction::VisitBlockFinish(VisitBlockFinish {
                                level,
                                level_seed,
                                block,
                                block_index,
                                next: VisitBlockFinishNext {
                                    fold_levels: self,
                                    level,
                                },
                            })
                        },
                        Some(LevelState::Flushed { .. }) =>
                            panic!("block is already flushed for BlockFinish"),
                    }
                },
            }
        }
    }

    pub enum Instruction<'s, B, S> {
        Done(Done<'s, B, S>),
        VisitLevel(VisitLevel<'s, B, S>),
        VisitBlockStart(VisitBlockStart<'s, B, S>),
        VisitItem(VisitItem<'s, B, S>),
        VisitBlockFinish(VisitBlockFinish<'s, B, S>),
    }

    pub struct VisitLevel<'s, B, S> {
        pub level: &'s sketch::Level,
        pub next: VisitLevelNext<'s, B, S>,
    }

    pub struct VisitLevelNext<'s, B, S> {
        fold_levels: FoldLevels<'s, B, S>,
        level: &'s sketch::Level,
    }

    impl<'s, B, S> VisitLevelNext<'s, B, S> {
        pub fn level_ready(self, level_seed: S) -> VisitBlockStart<'s, B, S> {
            VisitBlockStart {
                level: self.level,
                level_seed,
                block_index: 0,
                next: VisitBlockStartNext {
                    fold_levels: self.fold_levels,
                    level: self.level,
                    block_index: 0,
                },
            }
        }
    }

    pub struct VisitBlockStart<'s, B, S> {
        pub level: &'s sketch::Level,
        pub level_seed: S,
        pub block_index: usize,
        pub next: VisitBlockStartNext<'s, B, S>,
    }

    pub struct VisitBlockStartNext<'s, B, S> {
        fold_levels: FoldLevels<'s, B, S>,
        level: &'s sketch::Level,
        block_index: usize,
    }

    impl<'s, B, S> VisitBlockStartNext<'s, B, S> {
        pub fn block_ready(mut self, block: B, level_seed: S) -> FoldLevels<'s, B, S> {
            let state = &mut self.fold_levels.levels[self.level.index].state;
            assert!(state.is_none());
            *state = Some(LevelState::Active { level_seed, block, block_index: self.block_index, });
            self.fold_levels
        }
    }

    pub struct VisitItem<'s, B, S> {
        pub level: &'s sketch::Level,
        pub level_seed: S,
        pub block: B,
        pub block_index: usize,
        pub block_item_index: usize,
        pub next: VisitItemNext<'s, B, S>,
    }

    pub struct VisitItemNext<'s, B, S> {
        fold_levels: FoldLevels<'s, B, S>,
        level: &'s sketch::Level,
        block_index: usize,
    }

    impl<'s, B, S> VisitItemNext<'s, B, S> {
        pub fn item_ready(mut self, block: B, level_seed: S) -> FoldLevels<'s, B, S> {
            let state = &mut self.fold_levels.levels[self.level.index].state;
            assert!(state.is_none());
            *state = Some(LevelState::Active { level_seed, block, block_index: self.block_index, });
            self.fold_levels
        }
    }

    pub struct VisitBlockFinish<'s, B, S> {
        pub level: &'s sketch::Level,
        pub level_seed: S,
        pub block: B,
        pub block_index: usize,
        pub next: VisitBlockFinishNext<'s, B, S>,
    }

    pub struct VisitBlockFinishNext<'s, B, S> {
        fold_levels: FoldLevels<'s, B, S>,
        level: &'s sketch::Level,
    }

    impl<'s, B, S> VisitBlockFinishNext<'s, B, S> {
        pub fn block_flushed(mut self, level_seed: S) -> FoldLevels<'s, B, S> {
            let state = &mut self.fold_levels.levels[self.level.index].state;
            assert!(state.is_none());
            *state = Some(LevelState::Flushed { level_seed, });
            self.fold_levels
        }
    }

    pub struct Done<'s, B, S> {
        fold_levels: FoldLevels<'s, B, S>,
    }

    impl<'s, B, S> Done<'s, B, S> {
        pub fn levels_iter<'a>(&'a self) -> impl Iterator<Item = (&'s sketch::Level, &'a S)> {
            self.fold_levels
                .levels
                .iter()
                .filter_map(|fold_level| {
                    if let Some(LevelState::Flushed { ref level_seed, }) = fold_level.state {
                        Some((fold_level.level, level_seed))
                    } else {
                        None
                    }
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        sketch,
        plan,
        fold,
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

        assert_eq!(
            fold_identity(&sketch),
            vec![
                (&sketch.levels()[0], (sketch.levels()[0].items_count, sketch.levels()[0].blocks_count)),
                (&sketch.levels()[1], (sketch.levels()[1].items_count, sketch.levels()[1].blocks_count)),
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

        assert_eq!(
            fold_identity(&sketch),
            vec![
                (&sketch.levels()[0], (sketch.levels()[0].items_count, sketch.levels()[0].blocks_count)),
                (&sketch.levels()[1], (sketch.levels()[1].items_count, sketch.levels()[1].blocks_count)),
                (&sketch.levels()[2], (sketch.levels()[2].items_count, sketch.levels()[2].blocks_count)),
            ],
        );
    }

    fn fold_identity<'s>(sketch: &'s sketch::Tree) -> Vec<(&'s sketch::Level, (usize, usize))> {
        struct Block;
        let mut fold = fold::fold_levels(sketch);
        loop {
            fold = match fold.next() {
                fold::Instruction::Done(done) =>
                    return done.levels_iter().map(|value| (value.0, value.1.clone())).collect(),
                fold::Instruction::VisitLevel(fold::VisitLevel { next, .. }) => {
                    let fold::VisitBlockStart { level_seed, next, .. } =
                        next.level_ready((0, 0));
                    next.block_ready(Block, level_seed)
                },
                fold::Instruction::VisitBlockStart(fold::VisitBlockStart { level_seed, next, .. }) =>
                    next.block_ready(Block, level_seed),
                fold::Instruction::VisitItem(fold::VisitItem { level_seed: (items, blocks), block: Block, next, .. }) =>
                    next.item_ready(Block, (items + 1, blocks)),
                fold::Instruction::VisitBlockFinish(fold::VisitBlockFinish { level_seed: (items, blocks), block: Block, next, .. }) =>
                    next.block_flushed((items, blocks + 1)),
            }
        }
    }
}
