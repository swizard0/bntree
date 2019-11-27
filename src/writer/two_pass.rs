use std::{
    ops::Add,
    marker::PhantomData,
};

use super::{fold, sketch};

pub fn write_blocks<'s, B1, B2, O>(sketch: &'s sketch::Tree) -> WriteBlocks<'s, B1, B2, O> {
    WriteBlocks {
        inner: Pass::Markup(Markup {
            fold_levels: fold::fold_levels(sketch),
            _marker: PhantomData,
        }),
    }
}

pub struct WriteBlocks<'s, B1, B2, O> {
    inner: Pass<'s, B1, B2, O>,
}

enum Pass<'s, B1, B2, O> {
    Markup(Markup<'s, B1, B2, O>),
    Write(Write<'s, B2, B2, O>),
}

impl<'s, B1, B2, O> WriteBlocks<'s, B1, B2, O> {
    pub fn next(self) -> Instruction<'s, B1, B2, O> {
        match self.inner {
            Pass::Markup(markup) => match markup.fold_levels.next() {
                fold::Instruction::VisitLevel(fold::VisitLevel { level, next }) =>
                    Instruction::InitialLevelSize(InitialLevelSize {
                        level,
                        next: InitialLevelSizeNext {
                            fold_levels_next: next,
                            _marker: PhantomData,
                        },
                    }),
                fold::Instruction::VisitBlockStart(fold::VisitBlockStart { level, level_seed, next, .. }) =>
                    Instruction::AllocMarkupBlock(AllocMarkupBlock {
                        level,
                        next: AllocMarkupBlockNext {
                            fold_levels_next: next,
                            level_size: level_seed,
                            _marker: PhantomData,
                        },
                    }),
                fold::Instruction::VisitItem(fold::VisitItem { level, level_seed, block, next, .. }) =>
                    Instruction::WriteMarkupItem(WriteMarkupItem {
                        level,
                        block,
                        next: WriteMarkupItemNext {
                            fold_levels_next: next,
                            level_size: level_seed,
                            _marker: PhantomData,
                        },
                    }),
                fold::Instruction::VisitBlockFinish(fold::VisitBlockFinish { level, level_seed, block, next, .. }) =>
                    Instruction::FinishMarkupBlock(FinishMarkupBlock {
                        level,
                        block,
                        next: FinishMarkupBlockNext {
                            fold_levels_next: next,
                            level_size: level_seed,
                            _marker: PhantomData,
                        },
                    }),
                fold::Instruction::Done(done) =>
                    unimplemented!(),
            },
            Pass::Write(write) => match write.fold_levels.next() {
                fold::Instruction::VisitLevel(fold::VisitLevel { level, next }) =>
                    unimplemented!(),
                fold::Instruction::VisitBlockStart(fold::VisitBlockStart { .. }) => {
                    unimplemented!()
                }
                fold::Instruction::VisitItem(fold::VisitItem { .. }) =>
                    unimplemented!(),
                fold::Instruction::VisitBlockFinish(fold::VisitBlockFinish { .. }) => {
                    unimplemented!()
                }
                fold::Instruction::Done(done) =>
                    unimplemented!(),
            },
        }
    }
}

pub enum Instruction<'s, B1, B2, O> {
    InitialLevelSize(InitialLevelSize<'s, B1, B2, O>),
    AllocMarkupBlock(AllocMarkupBlock<'s, B1, B2, O>),
    WriteMarkupItem(WriteMarkupItem<'s, B1, B2, O>),
    FinishMarkupBlock(FinishMarkupBlock<'s, B1, B2, O>),
}

struct Markup<'s, B1, B2, O> {
    fold_levels: fold::FoldLevels<'s, B1, O>,
    _marker: PhantomData<B2>,
}

pub struct InitialLevelSize<'s, B1, B2, O> {
    pub level: &'s sketch::Level,
    pub next: InitialLevelSizeNext<'s, B1, B2, O>,
}

pub struct InitialLevelSizeNext<'s, B1, B2, O> {
    fold_levels_next: fold::VisitLevelNext<'s, B1, O>,
    _marker: PhantomData<B2>,
}

impl<'s, B1, B2, O> InitialLevelSizeNext<'s, B1, B2, O> {
    pub fn initial_level_size(self, initial_size: O) -> AllocMarkupBlock<'s, B1, B2, O> {
        let fold::VisitBlockStart { level, level_seed: level_size, next: fold_levels_next, .. } =
            self.fold_levels_next.level_ready(initial_size);
        AllocMarkupBlock {
            level,
            next: AllocMarkupBlockNext {
                fold_levels_next,
                level_size,
                _marker: PhantomData,
            },
        }
    }
}

pub struct AllocMarkupBlock<'s, B1, B2, O> {
    pub level: &'s sketch::Level,
    pub next: AllocMarkupBlockNext<'s, B1, B2, O>,
}

pub struct AllocMarkupBlockNext<'s, B1, B2, O> {
    fold_levels_next: fold::VisitBlockStartNext<'s, B1, O>,
    level_size: O,
    _marker: PhantomData<B2>,
}

impl<'s, B1, B2, O> AllocMarkupBlockNext<'s, B1, B2, O> {
    pub fn level_size(&self) -> &O {
        &self.level_size
    }

    pub fn block_ready(self, block: B1) -> WriteBlocks<'s, B1, B2, O> {
        WriteBlocks {
            inner: Pass::Markup(Markup {
                fold_levels: self.fold_levels_next.block_ready(block, self.level_size),
                _marker: PhantomData,
            }),
        }
    }
}

pub struct WriteMarkupItem<'s, B1, B2, O> {
    pub level: &'s sketch::Level,
    pub block: B1,
    pub next: WriteMarkupItemNext<'s, B1, B2, O>,
}

pub struct WriteMarkupItemNext<'s, B1, B2, O> {
    fold_levels_next: fold::VisitItemNext<'s, B1, O>,
    level_size: O,
    _marker: PhantomData<B2>,
}

impl<'s, B1, B2, O> WriteMarkupItemNext<'s, B1, B2, O> {
    pub fn level_size(&self) -> &O {
        &self.level_size
    }

    pub fn item_written(self, block: B1) -> WriteBlocks<'s, B1, B2, O> {
        WriteBlocks {
            inner: Pass::Markup(Markup {
                fold_levels: self.fold_levels_next.item_ready(block, self.level_size),
                _marker: PhantomData,
            }),
        }
    }
}

pub struct FinishMarkupBlock<'s, B1, B2, O> {
    pub level: &'s sketch::Level,
    pub block: B1,
    pub next: FinishMarkupBlockNext<'s, B1, B2, O>,
}

pub struct FinishMarkupBlockNext<'s, B1, B2, O> {
    fold_levels_next: fold::VisitBlockFinishNext<'s, B1, O>,
    level_size: O,
    _marker: PhantomData<B2>,
}

impl<'s, B1, B2, O> FinishMarkupBlockNext<'s, B1, B2, O> where O: Add<Output = O> {
    pub fn level_size(&self) -> &O {
        &self.level_size
    }

    pub fn block_finished(self, block_size: O) -> WriteBlocks<'s, B1, B2, O> {
        let level_size = self.level_size + block_size;
        WriteBlocks {
            inner: Pass::Markup(Markup {
                fold_levels: self.fold_levels_next.block_flushed(level_size),
                _marker: PhantomData,
            }),
        }
    }
}


struct Write<'s, B1, B2, O> {
    fold_levels: fold::FoldLevels<'s, B2, O>,
    _marker: PhantomData<B1>,
}

impl<'s, B1, B2, O> Write<'s, B1, B2, O> {
    pub fn next(self) -> Instruction<'s, B1, B2, O> {
        unimplemented!()
    }
}
