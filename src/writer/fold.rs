use super::{
    plan,
    super::sketch,
};

pub enum Instruction<S> {
    Perform(Perform<S>),
    Done,
}

pub struct Perform<S> {
    pub op: Op<S>,
    pub next_plan: plan::Instruction,
}

pub enum Op<S> {
    VisitLevel(VisitLevel),
    VisitBlockStart(VisitBlockStart<S>),
    VisitItem(VisitItem<S>),
    VisitBlockFinish(VisitBlockFinish<S>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    InvalidLevelStateForBlockStart,
    InvalidLevelStateForWriteItem,
    InvalidLevelStateForBlockFinish,
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
    InvalidLevelIndexForVisitBlockStart {
        level_index: usize,
    },
    AlreadyInitializedLevelForVisitBlockStart,
    InvalidLevelIndexForVisitItem {
        level_index: usize,
    },
    AlreadyInitializedLevelForVisitItem,
    InvalidLevelIndexForVisitBlockFinish {
        level_index: usize,
    },
    AlreadyInitializedLevelForVisitBlockFinish,
}

pub struct Script {
    inner: Fsm,
}

enum Fsm {
    Init,
    Busy,
}

#[derive(Default)]
pub struct Context<S> {
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


impl Script {
    pub fn new() -> Script {
        Script { inner: Fsm::Init, }
    }

    pub fn step<'s, S>(mut self, context: &mut Context<S>, arg: StepArg<'s>) -> Result<Instruction<S>, Error> {
        if let Fsm::Init = self.inner {
            context.levels.clear();
            context.levels.extend(
                arg.sketch
                    .levels()
                    .iter()
                    .map(|_level| Some(LevelState::Init))
            );
            self.inner = Fsm::Busy;
        }

        match arg.op {
            plan::Instruction::Perform(
                plan::Perform { op: plan::Op::BlockStart, level_index, block_index, next, }
            ) if level_index < context.levels.len() =>
                match context.levels[level_index].take() {
                    None =>
                        Err(Error::InvalidLevelStateForBlockStart),
                    Some(LevelState::Init) if block_index == 0 =>
                        Ok(Instruction::Perform(Perform {
                            op: Op::VisitLevel(VisitLevel {
                                level_index,
                                next: VisitLevelNext { level_index, script: self, },
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
                                    script: self,
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
            ) if level_index < context.levels.len() =>
                match context.levels[level_index].take() {
                    None =>
                        Err(Error::InvalidLevelStateForWriteItem),
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
                                    script: self,
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
            ) if level_index < context.levels.len() =>
                match context.levels[level_index].take() {
                    None =>
                        Err(Error::InvalidLevelStateForBlockFinish),
                    Some(LevelState::Init) =>
                        Err(Error::FinishingBlockBeforeBlockInitialize),
                    Some(LevelState::Active { level_seed, block_index: active_block_index, }) if active_block_index == block_index =>
                        Ok(Instruction::Perform(Perform {
                            op: Op::VisitBlockFinish(VisitBlockFinish {
                                level_index,
                                level_seed,
                                block_index,
                                next: VisitBlockFinishNext {
                                    script: self,
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
                Ok(Instruction::Done),
        }
    }
}


pub struct VisitLevel {
    pub level_index: usize,
    pub next: VisitLevelNext,
}

pub struct VisitLevelNext {
    level_index: usize,
    script: Script,
}

impl VisitLevelNext {
    pub fn level_ready<'s, S>(self, level_seed: S, _context: &mut Context<S>, arg: StepArg<'s>) -> Result<Instruction<S>, Error> {
        Ok(Instruction::Perform(Perform {
            op: Op::VisitBlockStart(VisitBlockStart {
                level_index: self.level_index,
                level_seed,
                block_index: 0,
                next: VisitBlockStartNext {
                    script: self.script,
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
    pub next: VisitBlockStartNext,
}

pub struct VisitBlockStartNext {
    script: Script,
    level_index: usize,
    block_index: usize,
}

impl VisitBlockStartNext {
    pub fn block_ready<'s, S>(self, level_seed: S, context: &mut Context<S>, arg: StepArg<'s>) -> Result<Instruction<S>, Error> {
        let state = context.levels.get_mut(self.level_index)
            .ok_or(Error::InvalidLevelIndexForVisitBlockStart { level_index: self.level_index, })?;
        if state.is_some() {
            return Err(Error::AlreadyInitializedLevelForVisitBlockStart);
        }
        *state = Some(LevelState::Active {
            level_seed,
            block_index: self.block_index,
        });
        self.script.step(context, arg)
    }
}


pub struct VisitItem<S> {
    pub level_index: usize,
    pub level_seed: S,
    pub block_index: usize,
    pub block_item_index: usize,
    pub next: VisitItemNext,
}

pub struct VisitItemNext {
    script: Script,
    level_index: usize,
    block_index: usize,
}

impl VisitItemNext {
    pub fn item_ready<'s, S>(self, level_seed: S, context: &mut Context<S>, arg: StepArg<'s>) -> Result<Instruction<S>, Error> {
        let state = context.levels.get_mut(self.level_index)
            .ok_or(Error::InvalidLevelIndexForVisitItem { level_index: self.level_index, })?;
        if state.is_some() {
            return Err(Error::AlreadyInitializedLevelForVisitItem);
        }
        *state = Some(LevelState::Active {
            level_seed,
            block_index: self.block_index,
        });
        self.script.step(context, arg)
    }
}


pub struct VisitBlockFinish<S> {
    pub level_index: usize,
    pub level_seed: S,
    pub block_index: usize,
    pub next: VisitBlockFinishNext,
}

pub struct VisitBlockFinishNext {
    script: Script,
    level_index: usize,
}

impl VisitBlockFinishNext {
    pub fn block_flushed<'s, S>(self, level_seed: S, context: &mut Context<S>, arg: StepArg<'s>) -> Result<Instruction<S>, Error> {
        let state = context.levels.get_mut(self.level_index)
            .ok_or(Error::InvalidLevelIndexForVisitBlockFinish { level_index: self.level_index, })?;
        if state.is_some() {
            return Err(Error::AlreadyInitializedLevelForVisitBlockFinish);
        }
        *state = Some(LevelState::Flushed { level_seed, });
        self.script.step(context, arg)
    }
}


impl<S> Context<S> {
    pub fn levels_iter(self) -> impl Iterator<Item = (usize, S)> {
        self.levels
            .into_iter()
            .enumerate()
            .filter_map(|(level_index, fold_level)| {
                if let Some(LevelState::Flushed { level_seed, }) = fold_level {
                    Some((level_index, level_seed))
                } else {
                    None
                }
            })
    }
}
