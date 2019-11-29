use std::{
    ops::Add,
    marker::PhantomData,
};

use super::{fold, sketch};

pub fn write_blocks<'s, B1, B2, O>(
    sketch: &'s sketch::Tree,
    tree_offset: O,
    tree_header_size: O,
)
    -> WriteBlocks<'s, B1, B2, O>
{
    WriteBlocks {
        inner: Pass::Markup(Markup {
            fold_levels: fold::fold_levels(sketch),
            _marker: PhantomData,
        }),
        coords: TreeCoords {
            offset: tree_offset,
            header_size: tree_header_size,
        },
    }
}

pub struct WriteBlocks<'s, B1, B2, O> {
    inner: Pass<'s, B1, B2, O>,
    coords: TreeCoords<O>,
}

enum Pass<'s, B1, B2, O> {
    Markup(Markup<'s, B1, B2, O>),
    Write(Write<'s, B1, B2, O>),
}

struct TreeCoords<O> {
    offset: O,
    header_size: O,
}

impl<'s, B1, B2, O> WriteBlocks<'s, B1, B2, O> where O: Add<Output = O> + Clone {
    pub fn next(self) -> Instruction<'s, B1, B2, O> {
        match self.inner {
            Pass::Markup(markup) => match markup.fold_levels.next() {
                fold::Instruction::VisitLevel(fold::VisitLevel { level, next }) =>
                    Instruction::InitialLevelSize(InitialLevelSize {
                        level,
                        next: InitialLevelSizeNext {
                            fold_levels_next: next,
                            tree_coords: self.coords,
                            _marker: PhantomData,
                        },
                    }),
                fold::Instruction::VisitBlockStart(fold::VisitBlockStart { level, level_seed, next, .. }) =>
                    Instruction::AllocMarkupBlock(AllocMarkupBlock {
                        level,
                        next: AllocMarkupBlockNext {
                            fold_levels_next: next,
                            tree_coords: self.coords,
                            level_seed,
                            _marker: PhantomData,
                        },
                    }),
                fold::Instruction::VisitItem(fold::VisitItem { level, level_seed, block, next, .. }) =>
                    Instruction::WriteMarkupItem(WriteMarkupItem {
                        level,
                        block,
                        next: WriteMarkupItemNext {
                            fold_levels_next: next,
                            tree_coords: self.coords,
                            level_seed,
                            _marker: PhantomData,
                        },
                    }),
                fold::Instruction::VisitBlockFinish(fold::VisitBlockFinish { level, level_seed, block, next, .. }) =>
                    Instruction::FinishMarkupBlock(FinishMarkupBlock {
                        level,
                        block,
                        next: FinishMarkupBlockNext {
                            fold_levels_next: next,
                            tree_coords: self.coords,
                            level_seed,
                            _marker: PhantomData,
                        },
                    }),
                fold::Instruction::Done(done) => {
                    let mut iter = done.levels_iter();
                    if let Some((_level, MarkupLevelSeed { level_size, .. })) = iter.next() {
                        let tree_total_size = iter.fold(
                            level_size.clone(),
                            |total, (_level, level_seed)| {
                                total + level_seed.level_size.clone()
                            },
                        );
                        Instruction::WriteTreeHeader(WriteTreeHeader {
                            offset: self.coords.offset.clone(),
                            tree_total_size,
                            next: WriteTreeHeaderNext {
                                levels_markup: done,
                                tree_coords: self.coords,
                                _marker: PhantomData,
                            },
                        })
                    } else {
                        Instruction::Done
                    }
                },
            },
            Pass::Write(write) => match write.fold_levels.next() {
                fold::Instruction::VisitLevel(fold::VisitLevel { level, next, }) => {
                    let level_offset = write
                        .levels_markup
                        .levels_iter()
                        .fold(
                            self.coords.header_size.clone(),
                            |offset, (_level, level_seed)| {
                                offset + level_seed.level_size.clone()
                            }
                        );
                    Instruction::WriteLevelHeader(WriteLevelHeader {
                        level,
                        level_offset,
                        next: WriteLevelHeaderNext {
                            fold_levels_next: next,
                            levels_markup: write.levels_markup,
                            tree_coords: self.coords,
                        },
                    })
                },
                fold::Instruction::VisitBlockStart(fold::VisitBlockStart { .. }) => {
                    unimplemented!()
                }
                fold::Instruction::VisitItem(fold::VisitItem { .. }) =>
                    unimplemented!(),
                fold::Instruction::VisitBlockFinish(fold::VisitBlockFinish { .. }) => {
                    unimplemented!()
                }
                fold::Instruction::Done(..) =>
                    Instruction::Done,
            },
        }
    }
}

pub enum Instruction<'s, B1, B2, O> {
    InitialLevelSize(InitialLevelSize<'s, B1, B2, O>),
    AllocMarkupBlock(AllocMarkupBlock<'s, B1, B2, O>),
    WriteMarkupItem(WriteMarkupItem<'s, B1, B2, O>),
    FinishMarkupBlock(FinishMarkupBlock<'s, B1, B2, O>),
    WriteTreeHeader(WriteTreeHeader<'s, B1, B2, O>),
    WriteLevelHeader(WriteLevelHeader<'s, B1, B2, O>),
    Done,
}

struct Markup<'s, B1, B2, O> {
    fold_levels: fold::FoldLevels<'s, B1, MarkupLevelSeed<O>>,
    _marker: PhantomData<B2>,
}

struct MarkupLevelSeed<O> {
    level_header_size: O,
    level_size: O,
}

pub struct InitialLevelSize<'s, B1, B2, O> {
    pub level: &'s sketch::Level,
    pub next: InitialLevelSizeNext<'s, B1, B2, O>,
}

pub struct InitialLevelSizeNext<'s, B1, B2, O> {
    fold_levels_next: fold::VisitLevelNext<'s, B1, MarkupLevelSeed<O>>,
    tree_coords: TreeCoords<O>,
    _marker: PhantomData<B2>,
}

impl<'s, B1, B2, O> InitialLevelSizeNext<'s, B1, B2, O> where O: Clone {
    pub fn level_header_size(self, level_header_size: O) -> AllocMarkupBlock<'s, B1, B2, O> {
        let fold::VisitBlockStart { level, level_seed, next: fold_levels_next, .. } =
            self.fold_levels_next.level_ready(MarkupLevelSeed {
                level_size: level_header_size.clone(),
                level_header_size,
            });
        AllocMarkupBlock {
            level,
            next: AllocMarkupBlockNext {
                fold_levels_next,
                tree_coords: self.tree_coords,
                level_seed,
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
    fold_levels_next: fold::VisitBlockStartNext<'s, B1, MarkupLevelSeed<O>>,
    tree_coords: TreeCoords<O>,
    level_seed: MarkupLevelSeed<O>,
    _marker: PhantomData<B2>,
}

impl<'s, B1, B2, O> AllocMarkupBlockNext<'s, B1, B2, O> {
    pub fn level_size(&self) -> &O {
        &self.level_seed.level_size
    }

    pub fn block_ready(self, block: B1) -> WriteBlocks<'s, B1, B2, O> {
        WriteBlocks {
            inner: Pass::Markup(Markup {
                fold_levels: self.fold_levels_next.block_ready(
                    block,
                    self.level_seed,
                ),
                _marker: PhantomData,
            }),
            coords: self.tree_coords,
        }
    }
}

pub struct WriteMarkupItem<'s, B1, B2, O> {
    pub level: &'s sketch::Level,
    pub block: B1,
    pub next: WriteMarkupItemNext<'s, B1, B2, O>,
}

pub struct WriteMarkupItemNext<'s, B1, B2, O> {
    fold_levels_next: fold::VisitItemNext<'s, B1, MarkupLevelSeed<O>>,
    tree_coords: TreeCoords<O>,
    level_seed: MarkupLevelSeed<O>,
    _marker: PhantomData<B2>,
}

impl<'s, B1, B2, O> WriteMarkupItemNext<'s, B1, B2, O> {
    pub fn level_size(&self) -> &O {
        &self.level_seed.level_size
    }

    pub fn item_written(self, block: B1) -> WriteBlocks<'s, B1, B2, O> {
        WriteBlocks {
            inner: Pass::Markup(Markup {
                fold_levels: self.fold_levels_next.item_ready(
                    block,
                    self.level_seed,
                ),
                _marker: PhantomData,
            }),
            coords: self.tree_coords,
        }
    }
}

pub struct FinishMarkupBlock<'s, B1, B2, O> {
    pub level: &'s sketch::Level,
    pub block: B1,
    pub next: FinishMarkupBlockNext<'s, B1, B2, O>,
}

pub struct FinishMarkupBlockNext<'s, B1, B2, O> {
    fold_levels_next: fold::VisitBlockFinishNext<'s, B1, MarkupLevelSeed<O>>,
    tree_coords: TreeCoords<O>,
    level_seed: MarkupLevelSeed<O>,
    _marker: PhantomData<B2>,
}

impl<'s, B1, B2, O> FinishMarkupBlockNext<'s, B1, B2, O> where O: Add<Output = O> {
    pub fn level_size(&self) -> &O {
        &self.level_seed.level_size
    }

    pub fn block_finished(self, block_size: O) -> WriteBlocks<'s, B1, B2, O> {
        let level_size = self.level_seed.level_size + block_size;
        WriteBlocks {
            inner: Pass::Markup(Markup {
                fold_levels: self.fold_levels_next.block_flushed(MarkupLevelSeed {
                    level_header_size: self.level_seed.level_header_size,
                    level_size,
                }),
                _marker: PhantomData,
            }),
            coords: self.tree_coords,
        }
    }
}


pub struct WriteTreeHeader<'s, B1, B2, O> {
    pub offset: O,
    pub tree_total_size: O,
    pub next: WriteTreeHeaderNext<'s, B1, B2, O>,
}

pub struct WriteTreeHeaderNext<'s, B1, B2, O> {
    levels_markup: fold::Done<'s, B1, MarkupLevelSeed<O>>,
    tree_coords: TreeCoords<O>,
    _marker: PhantomData<B2>,
}

impl<'s, B1, B2, O> WriteTreeHeaderNext<'s, B1, B2, O> {
    pub fn tree_header_written(self) -> WriteBlocks<'s, B1, B2, O> {
        WriteBlocks {
            inner: Pass::Write(Write {
                fold_levels: fold::fold_levels(self.levels_markup.sketch()),
                levels_markup: self.levels_markup,
            }),
            coords: self.tree_coords,
        }
    }
}


struct Write<'s, B1, B2, O> {
    fold_levels: fold::FoldLevels<'s, B2, O>,
    levels_markup: fold::Done<'s, B1, MarkupLevelSeed<O>>,
}


pub struct WriteLevelHeader<'s, B1, B2, O> {
    pub level: &'s sketch::Level,
    pub level_offset: O,
    pub next: WriteLevelHeaderNext<'s, B1, B2, O>,
}

pub struct WriteLevelHeaderNext<'s, B1, B2, O> {
    fold_levels_next: fold::VisitLevelNext<'s, B2, O>,
    levels_markup: fold::Done<'s, B1, MarkupLevelSeed<O>>,
    tree_coords: TreeCoords<O>,
}

impl<'s, B1, B2, O> WriteLevelHeaderNext<'s, B1, B2, O> {
    pub fn level_header_written(self) -> WriteBlocks<'s, B1, B2, O> {
        unimplemented!()
    }
}
