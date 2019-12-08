use super::{
    plan,
    super::sketch,
};

pub enum Instruction<S> {
    TreeStart { next: Script<S>, },
    Perform(Perform<S>),
    Done,
}

pub struct Perform<S> {
    pub op: Op<S>,
    pub next_plan: plan::Instruction,
}

pub enum Op<S> {
    VisitLevel(VisitLevel<S>),
    VisitBlockStart(VisitBlockStart<S>),
    VisitItem(VisitItem<S>),
    // VisitBlockFinish(VisitBlockFinish<S>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    NoExpectedPlanTreeStart,
    UnexpectedPlanTreeStart,
    InvalidPlanBlockStartLevelIndex,
    UnexpectedNonZeroBlockIndexForLevelStateInit,
    UnexpectedLevelStateActiveForPlanBlockStart {
        level_index: usize,
        block_index: usize,
        prev_block_index: usize,
    },
    UnexpectedZeroBlockIndexForLevelStateFlushed,
    InvalidPlanBlockItemLevelIndex,
    WritingItemBeforeBlockInitialize,
    WritingItemAfterBlockFlush,
    WritingItemInWrongBlock {
        block_index: usize,
        active_block_index: usize,
    },
}

pub struct Script<S> {
    inner: Fsm<S>,
}

enum Fsm<S> {
    Init,
    Busy(Busy<S>),
}

struct Busy<S> {
    levels: Vec<Option<LevelState<S>>>,
}

enum LevelState<S> {
    Init,
    Active {
        level_seed: S,
        block_index: usize,
    },
    Flushed {
        level_seed: S,
    },
}


impl<S> Script<S> {
    pub fn start() -> Instruction<S> {
        Instruction::TreeStart { next: Script { inner: Fsm::Init, } }
    }

    pub fn step(self, op: plan::Instruction, sketch: &sketch::Tree) -> Result<Instruction<S>, Error> {
        match self.inner {
            Fsm::Init =>
                if let plan::Instruction::Perform(plan::Perform { op: plan::Op::TreeStart, next, }) = op {
                    Busy {
                        levels: sketch
                            .levels()
                            .iter()
                            .map(|_level| Some(LevelState::Init))
                            .collect(),
                    }.step(next.step(sketch), sketch)
                } else {
                    Err(Error::NoExpectedPlanTreeStart)
                },
            Fsm::Busy(busy) =>
                busy.step(op, sketch),
        }
    }
}

impl<S> Busy<S> {
    fn step(mut self, op: plan::Instruction, sketch: &sketch::Tree) -> Result<Instruction<S>, Error> {
        match op {
            plan::Instruction::Perform(plan::Perform { op: plan::Op::TreeStart, .. }) =>
                Err(Error::UnexpectedPlanTreeStart),
            plan::Instruction::Perform(
                plan::Perform {
                    op: plan::Op::Block(plan::PerformBlock { op: plan::BlockOp::Start, level_index, block_index, }),
                    next,
                }
            ) if level_index < self.levels.len() =>
                match self.levels[level_index].take() {
                    None =>
                        panic!("level state left in invalid state for BlockStart"),
                    Some(LevelState::Init) if block_index == 0 =>
                        Ok(Instruction::Perform(Perform {
                            op: Op::VisitLevel(VisitLevel {
                                level_index,
                                next: VisitLevelNext { level_index, busy: self, },
                            }),
                            next_plan: next.step(sketch),
                        })),
                    Some(LevelState::Init) =>
                        Err(Error::UnexpectedNonZeroBlockIndexForLevelStateInit),
                    Some(LevelState::Active { block_index: prev_block_index, .. }) =>
                        Err(Error::UnexpectedLevelStateActiveForPlanBlockStart {
                            level_index,
                            block_index,
                            prev_block_index,
                        }),
                    Some(LevelState::Flushed { level_seed, }) if block_index > 0 =>
                        Ok(Instruction::Perform(Perform {
                            op: Op::VisitBlockStart(VisitBlockStart {
                                level_index,
                                level_seed,
                                block_index,
                                next: VisitBlockStartNext {
                                    busy: self,
                                    level_index,
                                    block_index,
                                },
                            }),
                            next_plan: next.step(sketch),
                        })),
                    Some(LevelState::Flushed { .. }) =>
                        Err(Error::UnexpectedZeroBlockIndexForLevelStateFlushed),
                },
            plan::Instruction::Perform(plan::Perform { op: plan::Op::Block(plan::PerformBlock { op: plan::BlockOp::Start, .. }), .. }) =>
                Err(Error::InvalidPlanBlockStartLevelIndex),

            plan::Instruction::Perform(
                plan::Perform {
                    op: plan::Op::Block(plan::PerformBlock { op: plan::BlockOp::Item { index: item_index, }, level_index, block_index, }),
                    next,
                },
            ) if level_index < self.levels.len() =>
                match self.levels[level_index].take() {
                    None =>
                        panic!("level state left in invalid state for WriteItem"),
                    Some(LevelState::Init) =>
                        Err(Error::WritingItemBeforeBlockInitialize),
                    Some(LevelState::Active { level_seed, block_index: active_block_index, }) if active_block_index == block_index =>
                        Ok(Instruction::Perform(Perform {
                            op: Op::VisitItem(VisitItem {
                                level_index,
                                level_seed,
                                block_index,
                                block_item_index: item_index,
                                next: VisitItemNext {
                                    busy: self,
                                    level_index,
                                    block_index,
                                },
                            }),
                            next_plan: next.step(sketch),
                        })),
                    Some(LevelState::Active { block_index: active_block_index, .. }) =>
                        Err(Error::WritingItemInWrongBlock { active_block_index, block_index, }),
                    Some(LevelState::Flushed { .. }) =>
                        Err(Error::WritingItemAfterBlockFlush),
                },
            plan::Instruction::Perform(plan::Perform { op: plan::Op::Block(plan::PerformBlock { op: plan::BlockOp::Item { .. }, .. }), .. }) =>
                Err(Error::InvalidPlanBlockItemLevelIndex),

            plan::Instruction::Perform(
                plan::Perform {
                    op: plan::Op::Block(plan::PerformBlock { op: plan::BlockOp::Finish, level_index, block_index, }),
                    next,
                }
            ) => {
                unimplemented!()
            },

            plan::Instruction::Done => {
                unimplemented!()
            },
        }
    }
}


pub struct VisitLevel<S> {
    pub level_index: usize,
    pub next: VisitLevelNext<S>,
}

pub struct VisitLevelNext<S> {
    level_index: usize,
    busy: Busy<S>,
}

impl<S> VisitLevelNext<S> {
    pub fn level_ready(self, level_seed: S, op: plan::Instruction, _sketch: &sketch::Tree) -> Result<Instruction<S>, Error> {
        Ok(Instruction::Perform(Perform {
            op: Op::VisitBlockStart(VisitBlockStart {
                level_index: self.level_index,
                level_seed,
                block_index: 0,
                next: VisitBlockStartNext {
                    busy: self.busy,
                    level_index: self.level_index,
                    block_index: 0,
                },
            }),
            next_plan: op,
        }))
    }
}


pub struct VisitBlockStart<S> {
    pub level_index: usize,
    pub level_seed: S,
    pub block_index: usize,
    pub next: VisitBlockStartNext<S>,
}

pub struct VisitBlockStartNext<S> {
    busy: Busy<S>,
    level_index: usize,
    block_index: usize,
}

impl<S> VisitBlockStartNext<S> {
    pub fn block_ready(mut self, level_seed: S, op: plan::Instruction, sketch: &sketch::Tree) -> Result<Instruction<S>, Error> {
        let state = &mut self.busy.levels[self.level_index];
        assert!(state.is_none());
        *state = Some(LevelState::Active {
            level_seed,
            block_index: self.block_index,
        });
        Script {
            inner: Fsm::Busy(self.busy),
        }.step(op, sketch)
    }
}


pub struct VisitItem<S> {
    pub level_index: usize,
    pub level_seed: S,
    pub block_index: usize,
    pub block_item_index: usize,
    pub next: VisitItemNext<S>,
}

pub struct VisitItemNext<S> {
    busy: Busy<S>,
    level_index: usize,
    block_index: usize,
}

impl<S> VisitItemNext<S> {
    pub fn item_ready(mut self, level_seed: S, op: plan::Instruction, sketch: &sketch::Tree) -> Result<Instruction<S>, Error> {
        let state = &mut self.busy.levels[self.level_index];
        assert!(state.is_none());
        *state = Some(LevelState::Active {
            level_seed,
            block_index: self.block_index,
        });
        Script {
            inner: Fsm::Busy(self.busy),
        }.step(op, sketch)
    }
}


// impl<'s, B, S> FoldLevels<'s, B, S> {
//     pub fn next(mut self) -> Instruction<'s, B, S> {
//         match self.plan.next() {
//             None =>
//                 Instruction::Done(Done { fold_levels: self, }),
//             Some(plan::Instruction { op: plan::Op::WriteItem { block_item_index }, level, block_index, .. }) => {
//                 assert!(level.index < self.levels.len());
//                 match self.levels[level.index].state.take() {
//                     None =>
//                         panic!("level state left in invalid state for WriteItem"),
//                     Some(LevelState::Init) =>
//                         panic!("level and block are not initialized for WriteItem"),
//                     Some(LevelState::Active { level_seed, block, block_index: active_block_index, }) => {
//                         assert!(active_block_index == block_index);
//                         Instruction::VisitItem(VisitItem {
//                             level,
//                             level_seed,
//                             block,
//                             block_index,
//                             block_item_index,
//                             next: VisitItemNext {
//                                 fold_levels: self,
//                                 level,
//                                 block_index,
//                             },
//                         })
//                     },
//                     Some(LevelState::Flushed { .. }) =>
//                         panic!("block is already flushed for WriteItem"),
//                 }
//             },
//             Some(plan::Instruction { op: plan::Op::BlockFinish, level, block_index, }) => {
//                 assert!(level.index < self.levels.len());
//                 match self.levels[level.index].state.take() {
//                     None =>
//                         panic!("level state left in invalid state for BlockFinish"),
//                     Some(LevelState::Init) =>
//                         panic!("level and block are not initialized for BlockFinish"),
//                     Some(LevelState::Active { level_seed, block, block_index: active_block_index, }) => {
//                         assert!(active_block_index == block_index);
//                         Instruction::VisitBlockFinish(VisitBlockFinish {
//                             level,
//                             level_seed,
//                             block,
//                             block_index,
//                             next: VisitBlockFinishNext {
//                                 fold_levels: self,
//                                 level,
//                             },
//                         })
//                     },
//                     Some(LevelState::Flushed { .. }) =>
//                         panic!("block is already flushed for BlockFinish"),
//                 }
//             },
//         }
//     }
// }

// pub enum Instruction<'s, B, S> {
//     Done(Done<'s, B, S>),
//     VisitLevel(VisitLevel<'s, B, S>),
//     VisitBlockStart(VisitBlockStart<'s, B, S>),
//     VisitItem(VisitItem<'s, B, S>),
//     VisitBlockFinish(VisitBlockFinish<'s, B, S>),
// }


// pub struct VisitItem<'s, B, S> {
//     pub level: &'s sketch::Level,
//     pub level_seed: S,
//     pub block: B,
//     pub block_index: usize,
//     pub block_item_index: usize,
//     pub next: VisitItemNext<'s, B, S>,
// }

// pub struct VisitItemNext<'s, B, S> {
//     fold_levels: FoldLevels<'s, B, S>,
//     level: &'s sketch::Level,
//     block_index: usize,
// }

// impl<'s, B, S> VisitItemNext<'s, B, S> {
//     pub fn item_ready(mut self, block: B, level_seed: S) -> FoldLevels<'s, B, S> {
//         let state = &mut self.fold_levels.levels[self.level.index].state;
//         assert!(state.is_none());
//         *state = Some(LevelState::Active {
//             level_seed,
//             block,
//             block_index: self.block_index,
//         });
//         self.fold_levels
//     }
// }

// pub struct VisitBlockFinish<'s, B, S> {
//     pub level: &'s sketch::Level,
//     pub level_seed: S,
//     pub block: B,
//     pub block_index: usize,
//     pub next: VisitBlockFinishNext<'s, B, S>,
// }

// pub struct VisitBlockFinishNext<'s, B, S> {
//     fold_levels: FoldLevels<'s, B, S>,
//     level: &'s sketch::Level,
// }

// impl<'s, B, S> VisitBlockFinishNext<'s, B, S> {
//     pub fn block_flushed(mut self, level_seed: S) -> FoldLevels<'s, B, S> {
//         let state = &mut self.fold_levels.levels[self.level.index].state;
//         assert!(state.is_none());
//         *state = Some(LevelState::Flushed { level_seed, });
//         self.fold_levels
//     }
// }

// pub struct Done<'s, B, S> {
//     fold_levels: FoldLevels<'s, B, S>,
// }

// impl<'s, B, S> Done<'s, B, S> {
//     pub fn sketch(&self) -> &'s sketch::Tree {
//         self.fold_levels.plan.sketch
//     }

//     pub fn levels_iter(self) -> impl Iterator<Item = (&'s sketch::Level, S)> {
//         self.fold_levels
//             .levels
//             .into_iter()
//             .filter_map(|fold_level| {
//                 if let Some(LevelState::Flushed { level_seed, }) = fold_level.state {
//                     Some((fold_level.level, level_seed))
//                 } else {
//                     None
//                 }
//             })
//     }
// }
