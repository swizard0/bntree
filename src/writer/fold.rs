use super::{plan, sketch};

pub fn fold_levels<'s, B, S>(sketch: &'s sketch::Tree) -> FoldLevels<'s, B, S> {
    FoldLevels {
        plan: plan::build(sketch),
        levels: sketch
            .levels()
            .iter()
            .map(|level| Level {
                level,
                state: Some(LevelState::Init),
            })
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
            Some(plan::Instruction { op: plan::Op::BlockStart, level, block_index, }) => {
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
            Some(plan::Instruction { op: plan::Op::WriteItem { block_item_index }, level, block_index, .. }) => {
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
            Some(plan::Instruction { op: plan::Op::BlockFinish, level, block_index, }) => {
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
        *state = Some(LevelState::Active {
            level_seed,
            block,
            block_index: self.block_index,
        });
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
        *state = Some(LevelState::Active {
            level_seed,
            block,
            block_index: self.block_index,
        });
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
    pub fn sketch(&self) -> &'s sketch::Tree {
        self.fold_levels.plan.sketch
    }

    pub fn levels_iter(self) -> impl Iterator<Item = (&'s sketch::Level, S)> {
        self.fold_levels
            .levels
            .into_iter()
            .filter_map(|fold_level| {
                if let Some(LevelState::Flushed { level_seed, }) = fold_level.state {
                    Some((fold_level.level, level_seed))
                } else {
                    None
                }
            })
    }
}
