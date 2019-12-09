use super::{
    plan,
    super::sketch,
};

pub enum Instruction<S> {
    Perform(Perform<S>),
    Done(Done<S>),
}

pub struct Perform<S> {
    pub op: Op<S>,
    pub next_plan: plan::Instruction,
}

pub enum Op<S> {
    VisitLevel(VisitLevel<S>),
    VisitBlockStart(VisitBlockStart<S>),
    VisitItem(VisitItem<S>),
    VisitBlockFinish(VisitBlockFinish<S>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Error {
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
    InvalidPlanBlockFinisLevelIndex,
    FinishingBlockBeforeBlockInitialize,
    FinishingTheWrongBlock {
        block_index: usize,
        active_block_index: usize,
    },
    FinishingBlockAfterBlockFlush,
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

pub struct StepArg<'s> {
    pub op: plan::Instruction,
    pub sketch: &'s sketch::Tree,
}


impl<S> Script<S> {
    pub fn start() -> Script<S> {
        Script { inner: Fsm::Init, }
    }

    pub fn step<'s>(self, arg: StepArg<'s>) -> Result<Instruction<S>, Error> {
        match self.inner {
            Fsm::Init =>
                Busy {
                    levels: arg.sketch
                        .levels()
                        .iter()
                        .map(|_level| Some(LevelState::Init))
                        .collect(),
                }.step(arg),
            Fsm::Busy(busy) =>
                busy.step(arg),
        }
    }
}

impl<S> Busy<S> {
    fn step<'s>(mut self, arg: StepArg<'s>) -> Result<Instruction<S>, Error> {
        match arg.op {
            plan::Instruction::Perform(
                plan::Perform { op: plan::Op::BlockStart, level_index, block_index, next, }
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
                            next_plan: next.step(arg.sketch),
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
                            next_plan: next.step(arg.sketch),
                        })),
                    Some(LevelState::Flushed { .. }) =>
                        Err(Error::UnexpectedZeroBlockIndexForLevelStateFlushed),
                },
            plan::Instruction::Perform(plan::Perform { op: plan::Op::BlockStart, .. }) =>
                Err(Error::InvalidPlanBlockStartLevelIndex),

            plan::Instruction::Perform(
                plan::Perform {
                    op: plan::Op::BlockItem { index: item_index, }, level_index, block_index, next,
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
                            next_plan: next.step(arg.sketch),
                        })),
                    Some(LevelState::Active { block_index: active_block_index, .. }) =>
                        Err(Error::WritingItemInWrongBlock { active_block_index, block_index, }),
                    Some(LevelState::Flushed { .. }) =>
                        Err(Error::WritingItemAfterBlockFlush),
                },
            plan::Instruction::Perform(plan::Perform { op: plan::Op::BlockItem { .. }, .. }) =>
                Err(Error::InvalidPlanBlockItemLevelIndex),

            plan::Instruction::Perform(
                plan::Perform { op: plan::Op::BlockFinish, level_index, block_index, next, }
            ) if level_index < self.levels.len() =>
                match self.levels[level_index].take() {
                    None =>
                        panic!("level state left in invalid state for BlockFinish"),
                    Some(LevelState::Init) =>
                        Err(Error::FinishingBlockBeforeBlockInitialize),
                    Some(LevelState::Active { level_seed, block_index: active_block_index, }) if active_block_index == block_index =>
                        Ok(Instruction::Perform(Perform {
                            op: Op::VisitBlockFinish(VisitBlockFinish {
                                level_index,
                                level_seed,
                                block_index,
                                next: VisitBlockFinishNext {
                                    busy: self,
                                    level_index,
                                },
                            }),
                            next_plan: next.step(arg.sketch),
                        })),
                    Some(LevelState::Active { block_index: active_block_index, .. }) =>
                        Err(Error::FinishingTheWrongBlock { active_block_index, block_index, }),
                    Some(LevelState::Flushed { .. }) =>
                        Err(Error::FinishingBlockAfterBlockFlush),
                }
            plan::Instruction::Perform(plan::Perform { op: plan::Op::BlockFinish, .. }) =>
                Err(Error::InvalidPlanBlockFinisLevelIndex),

            plan::Instruction::Done =>
                Ok(Instruction::Done(Done { busy: self, })),
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
    pub fn level_ready<'s>(self, level_seed: S, arg: StepArg<'s>) -> Result<Instruction<S>, Error> {
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
            next_plan: arg.op,
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
    pub fn block_ready<'s>(mut self, level_seed: S, arg: StepArg<'s>) -> Result<Instruction<S>, Error> {
        let state = &mut self.busy.levels[self.level_index];
        assert!(state.is_none());
        *state = Some(LevelState::Active {
            level_seed,
            block_index: self.block_index,
        });
        Script {
            inner: Fsm::Busy(self.busy),
        }.step(arg)
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
    pub fn item_ready<'s>(mut self, level_seed: S, arg: StepArg<'s>) -> Result<Instruction<S>, Error> {
        let state = &mut self.busy.levels[self.level_index];
        assert!(state.is_none());
        *state = Some(LevelState::Active {
            level_seed,
            block_index: self.block_index,
        });
        Script {
            inner: Fsm::Busy(self.busy),
        }.step(arg)
    }
}


pub struct VisitBlockFinish<S> {
    pub level_index: usize,
    pub level_seed: S,
    pub block_index: usize,
    pub next: VisitBlockFinishNext<S>,
}

pub struct VisitBlockFinishNext<S> {
    busy: Busy<S>,
    level_index: usize,
}

impl<S> VisitBlockFinishNext<S> {
    pub fn block_flushed<'s>(mut self, level_seed: S, arg: StepArg<'s>) -> Result<Instruction<S>, Error> {
        let state = &mut self.busy.levels[self.level_index];
        assert!(state.is_none());
        *state = Some(LevelState::Flushed { level_seed, });
        Script {
            inner: Fsm::Busy(self.busy),
        }.step(arg)
    }
}


pub struct Done<S> {
    busy: Busy<S>,
}

impl<S> Done<S> {
    pub fn levels_iter(self) -> impl Iterator<Item = (usize, S)> {
        self.busy
            .levels
            .into_iter()
            .filter_map(|fold_level| {
                if let Some(LevelState::Flushed { level_seed, }) = fold_level {
                    Some(level_seed)
                } else {
                    None
                }
            })
            .enumerate()
    }
}
