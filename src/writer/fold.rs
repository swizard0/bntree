use super::{
    plan,
    super::sketch,
};

pub enum Instruction<S> {
    Op(Op<S>),
    Done,
}

pub enum Op<S> {
    VisitLevel(VisitLevel),
    VisitBlockStart(VisitBlockStart<S>),
    VisitItem(VisitItem<S>),
    VisitBlockFinish(VisitBlockFinish<S>),
}

pub struct Continue {
    pub plan_action: PlanAction,
    pub next: Script,
}

pub enum PlanAction {
    Idle(plan::Instruction),
    Step(plan::Continue),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    UnexpectedNonZeroBlockIndexForLevelState,
    UnexpectedZeroBlockIndexForLevelState,
    InvalidLevelStateForBlockStart,
    InvalidLevelStateForWriteItem,
    InvalidLevelStateForBlockFinish,
    InvalidPlanBlockStartLevelIndex {
        level_index: usize,
    },
    UnexpectedLevelStateForPlanBlockStart {
        level_index: usize,
        block_index: usize,
        prev_block_index: usize,
    },
    InvalidPlanBlockItemLevelIndex {
        level_index: usize,
    },
    WritingItemBeforeLevelInitialize,
    WritingItemBeforeBlockInitialize,
    WritingItemAfterBlockFlush,
    WritingItemInWrongBlock {
        block_index: usize,
        active_block_index: usize,
    },
    InvalidPlanBlockFinisLevelIndex {
        level_index: usize,
    },
    FinishingBlockBeforeLevelInitialize,
    FinishingBlockBeforeBlockInitialize,
    FinishingTheWrongBlock {
        block_index: usize,
        active_block_index: usize,
    },
    FinishingBlockAfterBlockFlush,
    InvalidLevelIndexForVisitLevel {
        level_index: usize,
    },
    AlreadyInitializedLevelForVisitLevel,
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

pub struct Script(());

pub struct Context<S> {
    plan_ctx: plan::Context,
    levels: Vec<Option<LevelState<S>>>,
}

impl<S> Context<S> {
    pub fn new(plan_ctx: plan::Context, sketch: &sketch::Tree) -> Context<S> {
        Context {
            plan_ctx,
            levels: sketch
                .levels()
                .iter()
                .map(|_level| Some(LevelState::Bootstrap))
                .collect(),
        }
    }

    pub fn plan_ctx(&mut self) -> &mut plan::Context {
        &mut self.plan_ctx
    }
}

enum LevelState<S> {
    Bootstrap,
    SeenVisitLevel {
        level_seed: S,
    },
    SeenVisitBlockStart {
        level_seed: S,
        block_index: usize,
    },
    SeenVisitBlockFinish {
        level_seed: S,
    },
}

impl Script {
    pub fn boot() -> Continue {
        Continue {
            plan_action: PlanAction::Step(
                plan::Script::boot(),
            ),
            next: Script(()),
        }
    }

    pub fn step<S>(self, context: &mut Context<S>, op: plan::Instruction) -> Result<Instruction<S>, Error> {
        match op {
            plan::Instruction::Perform(plan::Perform { op: plan::Op::BlockStart, level_index, block_index, next, })  =>
                match context.levels.get_mut(level_index).map(Option::take) {
                    None =>
                        Err(Error::InvalidPlanBlockStartLevelIndex { level_index, }),
                    Some(None) =>
                        Err(Error::InvalidLevelStateForBlockStart),
                    Some(Some(LevelState::Bootstrap)) if block_index == 0 =>
                        Ok(Instruction::Op(Op::VisitLevel(VisitLevel {
                            level_index,
                            next: VisitLevelNext {
                                level_index,
                                script: self,
                                plan_next: plan::Instruction::Perform(plan::Perform {
                                    op: plan::Op::BlockStart,
                                    level_index: level_index,
                                    block_index: block_index,
                                    next,
                                }),
                            },
                        }))),
                    Some(Some(LevelState::Bootstrap)) =>
                        Err(Error::UnexpectedNonZeroBlockIndexForLevelState),
                    Some(Some(LevelState::SeenVisitLevel { level_seed, })) if block_index == 0 =>
                        Ok(Instruction::Op(Op::VisitBlockStart(VisitBlockStart {
                            level_index,
                            level_seed,
                            block_index: 0,
                            next: VisitBlockStartNext {
                                level_index,
                                block_index,
                                script: self,
                                plan_next: next,
                            },
                        }))),
                    Some(Some(LevelState::SeenVisitLevel { .. })) =>
                        Err(Error::UnexpectedNonZeroBlockIndexForLevelState),
                    Some(Some(LevelState::SeenVisitBlockStart { block_index: prev_block_index, .. })) =>
                        Err(Error::UnexpectedLevelStateForPlanBlockStart {
                            level_index,
                            block_index,
                            prev_block_index,
                        }),
                    Some(Some(LevelState::SeenVisitBlockFinish { level_seed, })) if block_index > 0 =>
                        Ok(Instruction::Op(Op::VisitBlockStart(VisitBlockStart {
                            level_index,
                            level_seed,
                            block_index,
                            next: VisitBlockStartNext {
                                level_index,
                                block_index,
                                script: self,
                                plan_next: next,
                            },
                        }))),
                    Some(Some(LevelState::SeenVisitBlockFinish { .. })) =>
                        Err(Error::UnexpectedZeroBlockIndexForLevelState),
                },

            plan::Instruction::Perform(
                plan::Perform {
                    op: plan::Op::BlockItem { index: item_index, }, level_index, block_index, next,
                },
            ) =>
                match context.levels.get_mut(level_index).map(Option::take) {
                    None =>
                        Err(Error::InvalidPlanBlockItemLevelIndex { level_index, }),
                    Some(None) =>
                        Err(Error::InvalidLevelStateForWriteItem),
                    Some(Some(LevelState::Bootstrap)) =>
                        Err(Error::WritingItemBeforeLevelInitialize),
                    Some(Some(LevelState::SeenVisitLevel { .. })) =>
                        Err(Error::WritingItemBeforeBlockInitialize),
                    Some(Some(LevelState::SeenVisitBlockStart { level_seed, block_index: active_block_index, }))
                        if active_block_index == block_index =>
                        Ok(Instruction::Op(Op::VisitItem(VisitItem {
                            level_index,
                            level_seed,
                            block_index,
                            block_item_index: item_index,
                            next: VisitItemNext {
                                level_index,
                                block_index,
                                script: self,
                                plan_next: next,
                            },
                        }))),
                    Some(Some(LevelState::SeenVisitBlockStart { block_index: active_block_index, .. })) =>
                        Err(Error::WritingItemInWrongBlock { active_block_index, block_index, }),
                    Some(Some(LevelState::SeenVisitBlockFinish { .. })) =>
                        Err(Error::WritingItemAfterBlockFlush),
                },

            plan::Instruction::Perform(
                plan::Perform { op: plan::Op::BlockFinish, level_index, block_index, next, }
            ) =>
                match context.levels.get_mut(level_index).map(Option::take) {
                    None =>
                        Err(Error::InvalidPlanBlockFinisLevelIndex { level_index, }),
                    Some(None) =>
                        Err(Error::InvalidLevelStateForBlockFinish),
                    Some(Some(LevelState::Bootstrap)) =>
                        Err(Error::FinishingBlockBeforeLevelInitialize),
                    Some(Some(LevelState::SeenVisitLevel { .. })) =>
                        Err(Error::FinishingBlockBeforeBlockInitialize),
                    Some(Some(LevelState::SeenVisitBlockStart { level_seed, block_index: active_block_index, }))
                        if active_block_index == block_index =>
                        Ok(Instruction::Op(Op::VisitBlockFinish(VisitBlockFinish {
                            level_index,
                            level_seed,
                            block_index,
                            next: VisitBlockFinishNext {
                                level_index,
                                script: self,
                                plan_next: next,
                            },
                        }))),
                    Some(Some(LevelState::SeenVisitBlockStart { block_index: active_block_index, .. })) =>
                        Err(Error::FinishingTheWrongBlock { active_block_index, block_index, }),
                    Some(Some(LevelState::SeenVisitBlockFinish { .. })) =>
                        Err(Error::FinishingBlockAfterBlockFlush),
                },

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
    plan_next: plan::Instruction,
}

impl VisitLevelNext {
    pub fn level_ready<S>(self, level_seed: S, context: &mut Context<S>) -> Result<Continue, Error> {
        let state = context.levels.get_mut(self.level_index)
            .ok_or(Error::InvalidLevelIndexForVisitLevel { level_index: self.level_index, })?;
        if state.is_some() {
            return Err(Error::AlreadyInitializedLevelForVisitLevel);
        }
        *state = Some(LevelState::SeenVisitLevel { level_seed, });
        Ok(Continue {
            plan_action: PlanAction::Idle(self.plan_next),
            next: self.script,
        })
    }
}


pub struct VisitBlockStart<S> {
    pub level_index: usize,
    pub level_seed: S,
    pub block_index: usize,
    pub next: VisitBlockStartNext,
}

pub struct VisitBlockStartNext {
    level_index: usize,
    block_index: usize,
    script: Script,
    plan_next: plan::Continue,
}

impl VisitBlockStartNext {
    pub fn block_ready<S>(self, level_seed: S, context: &mut Context<S>) -> Result<Continue, Error> {
        let state = context.levels.get_mut(self.level_index)
            .ok_or(Error::InvalidLevelIndexForVisitBlockStart { level_index: self.level_index, })?;
        if state.is_some() {
            return Err(Error::AlreadyInitializedLevelForVisitBlockStart);
        }
        *state = Some(LevelState::SeenVisitBlockStart {
            level_seed,
            block_index: self.block_index,
        });
        Ok(Continue {
            plan_action: PlanAction::Step(self.plan_next),
            next: self.script,
        })
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
    level_index: usize,
    block_index: usize,
    script: Script,
    plan_next: plan::Continue,
}

impl VisitItemNext {
    pub fn item_ready<S>(self, level_seed: S, context: &mut Context<S>) -> Result<Continue, Error> {
        let state = context.levels.get_mut(self.level_index)
            .ok_or(Error::InvalidLevelIndexForVisitItem { level_index: self.level_index, })?;
        if state.is_some() {
            return Err(Error::AlreadyInitializedLevelForVisitItem);
        }
        *state = Some(LevelState::SeenVisitBlockStart {
            level_seed,
            block_index: self.block_index,
        });
        Ok(Continue {
            plan_action: PlanAction::Step(self.plan_next),
            next: self.script,
        })
    }
}


pub struct VisitBlockFinish<S> {
    pub level_index: usize,
    pub level_seed: S,
    pub block_index: usize,
    pub next: VisitBlockFinishNext,
}

pub struct VisitBlockFinishNext {
    level_index: usize,
    script: Script,
    plan_next: plan::Continue,
}

impl VisitBlockFinishNext {
    pub fn block_flushed<S>(self, level_seed: S, context: &mut Context<S>) -> Result<Continue, Error> {
        let state = context.levels.get_mut(self.level_index)
            .ok_or(Error::InvalidLevelIndexForVisitBlockFinish { level_index: self.level_index, })?;
        if state.is_some() {
            return Err(Error::AlreadyInitializedLevelForVisitBlockFinish);
        }
        *state = Some(LevelState::SeenVisitBlockFinish { level_seed, });
        Ok(Continue {
            plan_action: PlanAction::Step(self.plan_next),
            next: self.script,
        })
    }
}


impl<S> Context<S> {
    pub fn into_levels_iter(self) -> (plan::Context, impl Iterator<Item = (usize, S)>) {
        (
            self.plan_ctx,
            self.levels
                .into_iter()
                .enumerate()
                .filter_map(|(level_index, fold_level)| {
                    if let Some(LevelState::SeenVisitBlockFinish { level_seed, }) = fold_level {
                        Some((level_index, level_seed))
                    } else {
                        None
                    }
                })
        )
    }
}
