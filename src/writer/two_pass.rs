use std::{
    ops::Add,
    marker::PhantomData,
};

use super::{
    fold,
    super::sketch,
};

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
                    let sketch = done.sketch();
                    let mut iter = done.levels_iter();
                    if let Some((level, markup)) = iter.next() {
                        let markup_car = LevelMarkup { level, markup, };
                        let mut markup_cdr = Vec::new();
                        let mut tree_total_size = self.coords.header_size.clone() +
                            markup_car.markup.level_size.clone();
                        for (level, markup) in iter {
                            tree_total_size = tree_total_size +
                                markup.level_size.clone();
                            markup_cdr.push(LevelMarkup { level, markup, });
                        }
                        Instruction::WriteTreeHeader(WriteTreeHeader {
                            offset: self.coords.offset.clone(),
                            tree_total_size,
                            next: WriteTreeHeaderNext {
                                sketch,
                                levels_markup: LevelsMarkup {
                                    markup_car,
                                    markup_cdr,
                                    _marker: PhantomData,
                                },
                                tree_coords: self.coords,
                                _marker: PhantomData,
                            },
                        })
                    } else {
                        Instruction::Done
                    }
                },
            },
            Pass::Write(mut write) => match write.fold_levels.next() {
                fold::Instruction::VisitLevel(fold::VisitLevel { level, next, }) => {
                    let mut level_offset = self.coords.header_size.clone();
                    let mut level_markup = &write.levels_markup.markup_car;
                    let mut level_header_size =
                        level_markup.markup.level_header_size.clone();
                    let mut iter = write.levels_markup.markup_cdr.iter();
                    loop {
                        if level_markup.level.index >= level.index {
                            break;
                        }
                        level_offset = level_offset +
                            level_markup.markup.level_size.clone();
                        level_header_size =
                            level_markup.markup.level_header_size.clone();
                        if let Some(markup) = iter.next() {
                            level_markup = markup;
                        } else {
                            break;
                        }
                    }
                    let block_offset =
                        level_offset.clone() + level_header_size.clone();
                    Instruction::WriteLevelHeader(WriteLevelHeader {
                        level,
                        level_offset,
                        next: WriteLevelHeaderNext {
                            fold_levels_next: next,
                            levels_markup: write.levels_markup,
                            recent_block_offset: write.recent_block_offset,
                            tree_coords: self.coords,
                            level_seed: WriteLevelSeed {
                                level_header_size,
                                cursor: block_offset.clone(),
                                block_offset,
                            },
                        },
                    })
                },
                fold::Instruction::VisitBlockStart(fold::VisitBlockStart { level, level_seed, next, .. }) =>
                    Instruction::WriteBlockHeader(WriteBlockHeader {
                        level,
                        block_offset: level_seed.block_offset.clone(),
                        next: WriteBlockHeaderNext {
                            fold_levels_next: next,
                            levels_markup: write.levels_markup,
                            recent_block_offset: write.recent_block_offset,
                            tree_coords: self.coords,
                            level_seed,
                        },
                    }),
                fold::Instruction::VisitItem(fold::VisitItem { level, level_seed, block, next, .. }) =>
                    Instruction::WriteBlockItem(WriteBlockItem {
                        level,
                        block_meta: block,
                        item_offset: level_seed.cursor.clone(),
                        child_block_offset: write.recent_block_offset.take(),
                        next: WriteBlockItemNext {
                            fold_levels_next: next,
                            levels_markup: write.levels_markup,
                            tree_coords: self.coords,
                            level_seed,
                        },
                    }),
                fold::Instruction::VisitBlockFinish(fold::VisitBlockFinish { level, level_seed, block, next, .. }) =>
                    Instruction::FlushBlock(FlushBlock {
                        level,
                        block_meta: block,
                        block_start_offset: level_seed.block_offset.clone(),
                        block_end_offset: level_seed.cursor.clone(),
                        next: FlushBlockNext {
                            fold_levels_next: next,
                            levels_markup: write.levels_markup,
                            tree_coords: self.coords,
                            level_seed,
                        },
                    }),
                fold::Instruction::Done(..) =>
                    Instruction::Done,
            },
        }
    }
}

// Instruction
pub enum Instruction<'s, B1, B2, O> {
    InitialLevelSize(InitialLevelSize<'s, B1, B2, O>),
    AllocMarkupBlock(AllocMarkupBlock<'s, B1, B2, O>),
    WriteMarkupItem(WriteMarkupItem<'s, B1, B2, O>),
    FinishMarkupBlock(FinishMarkupBlock<'s, B1, B2, O>),
    WriteTreeHeader(WriteTreeHeader<'s, B1, B2, O>),
    WriteLevelHeader(WriteLevelHeader<'s, B1, B2, O>),
    WriteBlockHeader(WriteBlockHeader<'s, B1, B2, O>),
    WriteBlockItem(WriteBlockItem<'s, B1, B2, O>),
    FlushBlock(FlushBlock<'s, B1, B2, O>),
    Done,
}

// Markup
struct Markup<'s, B1, B2, O> {
    fold_levels: fold::FoldLevels<'s, B1, MarkupLevelSeed<O>>,
    _marker: PhantomData<B2>,
}

struct MarkupLevelSeed<O> {
    level_header_size: O,
    level_size: O,
}

// InitialLevelSize
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

// AllocMarkupBlock
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

// WriteMarkupItem
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

// FinishMarkupBlock
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

// Write
struct Write<'s, B1, B2, O> {
    fold_levels: fold::FoldLevels<'s, B2, WriteLevelSeed<O>>,
    levels_markup: LevelsMarkup<'s, B1, O>,
    recent_block_offset: Option<O>,
}

struct WriteLevelSeed<O> {
    level_header_size: O,
    block_offset: O,
    cursor: O,
}

struct LevelMarkup<'s, O> {
    level: &'s sketch::Level,
    markup: MarkupLevelSeed<O>,
}

struct LevelsMarkup<'s, B1, O> {
    markup_car: LevelMarkup<'s, O>,
    markup_cdr: Vec<LevelMarkup<'s, O>>,
    _marker: PhantomData<B1>,
}

// WriteTreeHeader
pub struct WriteTreeHeader<'s, B1, B2, O> {
    pub offset: O,
    pub tree_total_size: O,
    pub next: WriteTreeHeaderNext<'s, B1, B2, O>,
}

pub struct WriteTreeHeaderNext<'s, B1, B2, O> {
    sketch: &'s sketch::Tree,
    levels_markup: LevelsMarkup<'s, B1, O>,
    tree_coords: TreeCoords<O>,
    _marker: PhantomData<B2>,
}

impl<'s, B1, B2, O> WriteTreeHeaderNext<'s, B1, B2, O> {
    pub fn tree_header_written(self, written: O) -> WriteBlocks<'s, B1, B2, O> where O: PartialEq {
        assert!(written == self.tree_coords.header_size);
        WriteBlocks {
            inner: Pass::Write(Write {
                fold_levels: fold::fold_levels(self.sketch),
                levels_markup: self.levels_markup,
                recent_block_offset: None,
            }),
            coords: self.tree_coords,
        }
    }
}

// WriteLevelHeader
pub struct WriteLevelHeader<'s, B1, B2, O> {
    pub level: &'s sketch::Level,
    pub level_offset: O,
    pub next: WriteLevelHeaderNext<'s, B1, B2, O>,
}

pub struct WriteLevelHeaderNext<'s, B1, B2, O> {
    fold_levels_next: fold::VisitLevelNext<'s, B2, WriteLevelSeed<O>>,
    levels_markup: LevelsMarkup<'s, B1, O>,
    recent_block_offset: Option<O>,
    tree_coords: TreeCoords<O>,
    level_seed: WriteLevelSeed<O>,
}

impl<'s, B1, B2, O> WriteLevelHeaderNext<'s, B1, B2, O> {
    pub fn level_header_written(self, written: O) -> WriteBlockHeader<'s, B1, B2, O> where O: Clone + PartialEq + Add<Output = O> {
        assert!(written == self.level_seed.level_header_size);
        let fold::VisitBlockStart { level, level_seed, next: fold_levels_next, .. } =
            self.fold_levels_next.level_ready(self.level_seed);
        WriteBlockHeader {
            level,
            block_offset: level_seed.block_offset.clone(),
            next: WriteBlockHeaderNext {
                fold_levels_next,
                levels_markup: self.levels_markup,
                recent_block_offset: self.recent_block_offset,
                tree_coords: self.tree_coords,
                level_seed,
            },
        }
    }
}

// WriteBlockHeader
pub struct WriteBlockHeader<'s, B1, B2, O> {
    pub level: &'s sketch::Level,
    pub block_offset: O,
    pub next: WriteBlockHeaderNext<'s, B1, B2, O>,
}

pub struct WriteBlockHeaderNext<'s, B1, B2, O> {
    fold_levels_next: fold::VisitBlockStartNext<'s, B2, WriteLevelSeed<O>>,
    levels_markup: LevelsMarkup<'s, B1, O>,
    recent_block_offset: Option<O>,
    tree_coords: TreeCoords<O>,
    level_seed: WriteLevelSeed<O>,
}

impl<'s, B1, B2, O> WriteBlockHeaderNext<'s, B1, B2, O> {
    pub fn block_header_written(self, block_meta: B2, written: O) -> WriteBlocks<'s, B1, B2, O> where O: Add<Output = O> {
        WriteBlocks {
            inner: Pass::Write(Write {
                fold_levels: self.fold_levels_next.block_ready(
                    block_meta,
                    WriteLevelSeed {
                        cursor: self.level_seed.cursor + written,
                        ..self.level_seed
                    },
                ),
                levels_markup: self.levels_markup,
                recent_block_offset: self.recent_block_offset,
            }),
            coords: self.tree_coords,
        }
    }
}

// WriteBlockItem
pub struct WriteBlockItem<'s, B1, B2, O> {
    pub level: &'s sketch::Level,
    pub block_meta: B2,
    pub item_offset: O,
    pub child_block_offset: Option<O>,
    pub next: WriteBlockItemNext<'s, B1, B2, O>,
}

pub struct WriteBlockItemNext<'s, B1, B2, O> {
    fold_levels_next: fold::VisitItemNext<'s, B2, WriteLevelSeed<O>>,
    levels_markup: LevelsMarkup<'s, B1, O>,
    tree_coords: TreeCoords<O>,
    level_seed: WriteLevelSeed<O>,
}

impl<'s, B1, B2, O> WriteBlockItemNext<'s, B1, B2, O> {
    pub fn item_written(self, block_meta: B2, written: O) -> WriteBlocks<'s, B1, B2, O> where O: Add<Output = O> {
        WriteBlocks {
            inner: Pass::Write(Write {
                fold_levels: self.fold_levels_next.item_ready(
                    block_meta,
                    WriteLevelSeed {
                        cursor: self.level_seed.cursor + written,
                        ..self.level_seed
                    },
                ),
                levels_markup: self.levels_markup,
                recent_block_offset: None,
            }),
            coords: self.tree_coords,
        }
    }
}

// FlushBlock
pub struct FlushBlock<'s, B1, B2, O> {
    pub level: &'s sketch::Level,
    pub block_meta: B2,
    pub block_start_offset: O,
    pub block_end_offset: O,
    pub next: FlushBlockNext<'s, B1, B2, O>,
}

pub struct FlushBlockNext<'s, B1, B2, O> {
    fold_levels_next: fold::VisitBlockFinishNext<'s, B2, WriteLevelSeed<O>>,
    levels_markup: LevelsMarkup<'s, B1, O>,
    tree_coords: TreeCoords<O>,
    level_seed: WriteLevelSeed<O>,
}

impl<'s, B1, B2, O> FlushBlockNext<'s, B1, B2, O> {
    pub fn block_flushed(self) -> WriteBlocks<'s, B1, B2, O> where O: Clone {
        let recent_block_offset =
            Some(self.level_seed.block_offset.clone());
        WriteBlocks {
            inner: Pass::Write(Write {
                fold_levels: self.fold_levels_next.block_flushed(
                    WriteLevelSeed {
                        block_offset: self.level_seed.cursor.clone(),
                        ..self.level_seed
                    },
                ),
                levels_markup: self.levels_markup,
                recent_block_offset,
            }),
            coords: self.tree_coords,
        }
    }
}
