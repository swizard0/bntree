#[derive(Clone, PartialEq, Debug)]
pub struct LevelCoords<O> {
    pub index: usize,
    pub header_size: O,
    pub total_size: O,
}

pub mod markup {
    use std::ops::Add;

    use super::super::fold;

    pub enum Instruction<B, O> {
        Op(Op<B, O>),
        Done,
    }

    pub enum Op<B, O> {
        LevelHeaderSize(LevelHeaderSize),
        AllocBlock(AllocBlock<B, O>),
        WriteItem(WriteItem<B, O>),
        FinishBlock(FinishBlock<B, O>),
    }

    pub struct Continue<B, O> {
        pub fold_action: FoldAction<B, O>,
        pub next: Script,
    }

    pub enum FoldAction<B, O> {
        Idle(fold::Instruction<LevelSeed<B, O>>),
        Step(fold::Continue),
    }

    #[derive(Clone, PartialEq, Debug)]
    pub enum Error {
        LevelHeaderSize(fold::Error),
        BlockReady(fold::Error),
        WriteItem(fold::Error),
        FinishBlock(fold::Error),
        NoActiveBlockWhileWriteItem,
        NoActiveBlockWhileBlockFinish,
        StepRec(fold::Error),
    }

    pub struct Context<B, O> {
        fold_ctx: fold::Context<LevelSeed<B, O>>,
        child_pending: bool,
    }

    impl<B, O> Context<B, O> {
        pub fn new(fold_ctx: fold::Context<LevelSeed<B, O>>) -> Context<B, O> {
            Context {
                fold_ctx,
                child_pending: false,
            }
        }

        pub fn fold_ctx(&mut self) -> &mut fold::Context<LevelSeed<B, O>> {
            &mut self.fold_ctx
        }
    }

    pub struct Script(());

    impl Script {
        pub fn boot<B, O>() -> Continue<B, O> {
            Continue {
                fold_action: FoldAction::Step(
                    fold::Script::boot(),
                ),
                next: Script(()),
            }
        }

        pub fn step<B, O>(self, context: &mut Context<B, O>, op: fold::Instruction<LevelSeed<B, O>>) -> Result<Instruction<B, O>, Error> {
            match op {
                fold::Instruction::Op(fold::Op::VisitLevel(fold::VisitLevel { level_index, next, })) =>
                    Ok(Instruction::Op(Op::LevelHeaderSize(LevelHeaderSize {
                        level_index,
                        next: LevelHeaderSizeNext {
                            script: self,
                            fold_next: next,
                        },
                    }))),
                fold::Instruction::Op(fold::Op::VisitBlockStart(fold::VisitBlockStart { level_index, level_seed, block_index, items_count, next, })) =>
                    Ok(Instruction::Op(Op::AllocBlock(AllocBlock {
                        level_index,
                        block_index,
                        items_count,
                        next: AllocBlockNext {
                            level_seed,
                            script: self,
                            fold_next: next,
                        },
                    }))),
                fold::Instruction::Op(fold::Op::VisitItem(fold::VisitItem {
                    level_index, mut level_seed, block_index, block_item_index, next,
                })) =>
                    if let Some(block) = level_seed.active_block.take() {
                        Ok(Instruction::Op(Op::WriteItem(WriteItem {
                            level_index,
                            block_index,
                            block_item_index,
                            block,
                            child_pending: context.child_pending,
                            next: WriteItemNext {
                                level_seed,
                                script: self,
                                fold_next: next,
                            },
                        })))
                    } else {
                        Err(Error::NoActiveBlockWhileWriteItem)
                    },
                fold::Instruction::Op(fold::Op::VisitBlockFinish(fold::VisitBlockFinish {
                    level_index, mut level_seed, block_index, next,
                })) =>
                    if let Some(block) = level_seed.active_block.take() {
                        Ok(Instruction::Op(Op::FinishBlock(FinishBlock {
                            level_index,
                            block_index,
                            block,
                            next: FinishBlockNext {
                                level_seed,
                                script: self,
                                fold_next: next,
                            },
                        })))
                    } else {
                        Err(Error::NoActiveBlockWhileBlockFinish)
                    },
                fold::Instruction::Done =>
                    Ok(Instruction::Done),
            }
        }
    }

    pub struct LevelSeed<B, O> {
        level_header_size: O,
        level_size: O,
        active_block: Option<B>,
    }


    pub struct LevelHeaderSize {
        pub level_index: usize,
        pub next: LevelHeaderSizeNext,
    }

    pub struct LevelHeaderSizeNext {
        script: Script,
        fold_next: fold::VisitLevelNext,
    }

    impl LevelHeaderSizeNext {
        pub fn level_header_size<B, O>(self, level_header_size: O, context: &mut Context<B, O>) -> Result<Continue<B, O>, Error>
        where O: Clone
        {
            let fold_continue = self
                .fold_next.level_ready(
                    LevelSeed {
                        level_size: level_header_size.clone(),
                        level_header_size,
                        active_block: None,
                    },
                    context.fold_ctx(),
                )
                .map_err(Error::LevelHeaderSize)?;
            Ok(Continue {
                fold_action: FoldAction::Step(fold_continue),
                next: self.script,
            })
        }
    }


    pub struct AllocBlock<B, O> {
        pub level_index: usize,
        pub block_index: usize,
        pub items_count: usize,
        pub next: AllocBlockNext<B, O>,
    }

    pub struct AllocBlockNext<B, O> {
        level_seed: LevelSeed<B, O>,
        script: Script,
        fold_next: fold::VisitBlockStartNext,
    }

    impl<B, O> AllocBlockNext<B, O> {
        pub fn block_ready(self, block: B, context: &mut Context<B, O>) -> Result<Continue<B, O>, Error>
        where O: Add<Output = O>
        {
            let fold_continue = self
                .fold_next.block_ready(
                    LevelSeed {
                        active_block: Some(block),
                        ..self.level_seed
                    },
                    context.fold_ctx(),
                )
                .map_err(Error::LevelHeaderSize)?;
            Ok(Continue {
                fold_action: FoldAction::Step(fold_continue),
                next: self.script,
            })
        }
    }


    pub struct WriteItem<B, O> {
        pub level_index: usize,
        pub block_index: usize,
        pub block_item_index: usize,
        pub block: B,
        pub child_pending: bool,
        pub next: WriteItemNext<B, O>,
    }

    pub struct WriteItemNext<B, O> {
        level_seed: LevelSeed<B, O>,
        script: Script,
        fold_next: fold::VisitItemNext,
    }

    impl<B, O> WriteItemNext<B, O> {
        pub fn item_written(self, block: B, context: &mut Context<B, O>) -> Result<Continue<B, O>, Error> {
            let fold_continue = self
                .fold_next.item_ready(
                    LevelSeed {
                        active_block: Some(block),
                        ..self.level_seed
                    },
                    context.fold_ctx(),
                )
                .map_err(Error::WriteItem)?;
            context.child_pending = false;
            Ok(Continue {
                fold_action: FoldAction::Step(fold_continue),
                next: self.script,
            })
        }
    }


    pub struct FinishBlock<B, O> {
        pub level_index: usize,
        pub block_index: usize,
        pub block: B,
        pub next: FinishBlockNext<B, O>,
    }

    pub struct FinishBlockNext<B, O> {
        level_seed: LevelSeed<B, O>,
        script: Script,
        fold_next: fold::VisitBlockFinishNext,
    }

    impl<B, O> FinishBlockNext<B, O> {
        pub fn block_finished(self, block_size: O, context: &mut Context<B, O>) -> Result<Continue<B, O>, Error>
        where O: Add<Output = O>,
        {
            let fold_continue = self
                .fold_next.block_flushed(
                    LevelSeed {
                        active_block: None,
                        level_size: self.level_seed.level_size + block_size,
                        ..self.level_seed
                    },
                    context.fold_ctx(),
                )
                .map_err(Error::FinishBlock)?;
            context.child_pending = true;
            Ok(Continue {
                fold_action: FoldAction::Step(fold_continue),
                next: self.script,
            })
        }
    }


    impl<B, O> Context<B, O> {
        pub fn into_level_coords_iter(self) -> impl Iterator<Item = super::LevelCoords<O>> {
            let (_plan_ctx, fold_iter) =
                self.fold_ctx.into_levels_iter();
            fold_iter
                .map(|(index, LevelSeed { level_header_size, level_size, .. })| {
                    super::LevelCoords {
                        index,
                        header_size: level_header_size,
                        total_size: level_size,
                    }
                })
        }
    }


    impl<B, O> Continue<B, O> {
        pub fn step_rec(self, markup_context: &mut Context<B, O>) -> Result<Instruction<B, O>, Error> {
            match self {
                Continue { fold_action: FoldAction::Idle(fold_op), next: markup_next, } =>
                    markup_next.step(markup_context, fold_op),
                Continue { fold_action: FoldAction::Step(fold_continue), next: markup_next, } => {
                    let fold_op = fold_continue
                        .step_rec(markup_context.fold_ctx())
                        .map_err(Error::StepRec)?;
                    markup_next.step(markup_context, fold_op)
                },
            }
        }
    }
}

pub mod write {
    use std::ops::Add;

    use super::{
        super::fold,
        LevelCoords,
    };

    pub enum Instruction<B, O> {
        Op(Op<B, O>),
        Done,
    }

    pub enum Op<B, O> {
        WriteTreeHeader(WriteTreeHeader<B, O>),
        WriteLevelHeader(WriteLevelHeader<B, O>),
        WriteBlockHeader(WriteBlockHeader<B, O>),
        WriteItem(WriteItem<B, O>),
        FlushBlock(FlushBlock<B, O>),
    }

    pub struct Continue<B, O> {
        pub fold_action: FoldAction<B, O>,
        pub next: Script,
    }

    pub enum FoldAction<B, O> {
        Idle(fold::Instruction<LevelSeed<B, O>>),
        Step(fold::Continue),
    }

    #[derive(Clone, PartialEq, Debug)]
    pub enum Error<O> {
        LevelsCoordsEmpty,
        WriteTreeHeaderSizeMismatch {
            written: O,
            expected: O,
        },
        WriteLevelHeader(fold::Error),
        WriteLevelHeaderSizeMismatch {
            written: O,
            expected: O,
        },
        WriteBlockHeader(fold::Error),
        NoActiveBlockWhileWriteItem,
        WriteItem(fold::Error),
        NoActiveBlockWhileFlushBlock,
        FlushBlock(fold::Error),
        StepRec(fold::Error),
    }

    pub struct Context<B, O> {
        fold_ctx: fold::Context<LevelSeed<B, O>>,
        tree_offset: O,
        tree_header_size: O,
        tree_header_written: bool,
        recent_block_offset: Option<O>,
        levels_car: LevelCoords<O>,
        levels_cdr: Vec<LevelCoords<O>>,
    }

    impl<B, O> Context<B, O> {
        pub fn new<I>(
            fold_ctx: fold::Context<LevelSeed<B, O>>,
            tree_offset: O,
            tree_header_size: O,
            mut levels_coords: I,
        )
            -> Result<Context<B, O>, Error<O>>
        where I: Iterator<Item = LevelCoords<O>>
        {
            if let Some(levels_car) = levels_coords.next() {
                Ok(Context {
                    fold_ctx,
                    tree_offset,
                    tree_header_size,
                    tree_header_written: false,
                    recent_block_offset: None,
                    levels_car,
                    levels_cdr: levels_coords.collect(),
                })
            } else {
                Err(Error::LevelsCoordsEmpty)
            }
        }

        pub fn fold_ctx(&mut self) -> &mut fold::Context<LevelSeed<B, O>> {
            &mut self.fold_ctx
        }
    }

    pub struct Script(());

    impl Script {
        pub fn boot<B, O>() -> Continue<B, O> {
            Continue {
                fold_action: FoldAction::Step(
                    fold::Script::boot(),
                ),
                next: Script(()),
            }
        }

        pub fn step<B, O>(self, context: &mut Context<B, O>, op: fold::Instruction<LevelSeed<B, O>>) -> Result<Instruction<B, O>, Error<O>>
        where O: Clone + Add<Output = O>,
        {
            if !context.tree_header_written {
                let tree_total_size = context
                    .levels_cdr
                    .iter()
                    .fold(
                        context.tree_header_size.clone() + context.levels_car.total_size.clone(),
                        |size, level| size + level.total_size.clone(),
                    );
                return Ok(Instruction::Op(Op::WriteTreeHeader(WriteTreeHeader {
                    tree_offset: context.tree_offset.clone(),
                    tree_header_size: context.tree_header_size.clone(),
                    tree_total_size: tree_total_size,
                    next: WriteTreeHeaderNext {
                        script: self,
                        fold_next: op,
                    },
                })));
            }
            match op {
                fold::Instruction::Op(fold::Op::VisitLevel(fold::VisitLevel { level_index, next, })) => {
                    let mut level_offset =
                        context.tree_header_size.clone() + context.tree_offset.clone();
                    let mut level_markup = &context.levels_car;
                    let mut level_header_size = level_markup.header_size.clone();
                    let mut iter = context.levels_cdr.iter();
                    loop {
                        if level_markup.index >= level_index {
                            break;
                        }
                        level_offset = level_offset + level_markup.total_size.clone();
                        level_header_size = level_markup.header_size.clone();
                        if let Some(markup) = iter.next() {
                            level_markup = markup;
                        } else {
                            break;
                        }
                    }
                    let block_offset = level_offset.clone() + level_header_size.clone();
                    Ok(Instruction::Op(Op::WriteLevelHeader(WriteLevelHeader {
                        level_index,
                        level_offset,
                        next: WriteLevelHeaderNext {
                            level_seed: LevelSeed {
                                level_header_size,
                                cursor: block_offset.clone(),
                                block_offset,
                                active_block: None,
                            },
                            script: self,
                            fold_next: next,
                        },
                    })))
                },
                fold::Instruction::Op(fold::Op::VisitBlockStart(fold::VisitBlockStart { level_index, level_seed, block_index, items_count, next, })) =>
                    Ok(Instruction::Op(Op::WriteBlockHeader(WriteBlockHeader {
                        level_index,
                        block_index,
                        block_offset: level_seed.block_offset.clone(),
                        items_count,
                        next: WriteBlockHeaderNext {
                            level_seed,
                            script: self,
                            fold_next: next,
                        },
                    }))),
                fold::Instruction::Op(fold::Op::VisitItem(fold::VisitItem {
                    level_index, mut level_seed, block_index, block_item_index, next,
                })) =>
                    if let Some(block) = level_seed.active_block.take() {
                        Ok(Instruction::Op(Op::WriteItem(WriteItem {
                            level_index,
                            block_index,
                            block_item_index,
                            block_meta: block,
                            child_block_offset: context.recent_block_offset.take(),
                            next: WriteItemNext {
                                level_seed,
                                script: self,
                                fold_next: next,
                            },
                        })))
                    } else {
                        Err(Error::NoActiveBlockWhileWriteItem)
                    },
                fold::Instruction::Op(fold::Op::VisitBlockFinish(fold::VisitBlockFinish {
                    level_index, mut level_seed, block_index, next,
                })) =>
                    if let Some(block) = level_seed.active_block.take() {
                        Ok(Instruction::Op(Op::FlushBlock(FlushBlock {
                            level_index,
                            block_index,
                            block_meta: block,
                            block_start_offset: level_seed.block_offset.clone(),
                            block_end_offset: level_seed.cursor.clone(),
                            next: FlushBlockNext {
                                level_seed,
                                script: self,
                                fold_next: next,
                            },
                        })))
                    } else {
                        Err(Error::NoActiveBlockWhileFlushBlock)
                    },
                fold::Instruction::Done =>
                    Ok(Instruction::Done),
            }
        }
    }

    pub struct LevelSeed<B, O> {
        level_header_size: O,
        block_offset: O,
        cursor: O,
        active_block: Option<B>,
    }


    pub struct WriteTreeHeader<B, O> {
        pub tree_offset: O,
        pub tree_header_size: O,
        pub tree_total_size: O,
        pub next: WriteTreeHeaderNext<B, O>,
    }

    pub struct WriteTreeHeaderNext<B, O> {
        script: Script,
        fold_next: fold::Instruction<LevelSeed<B, O>>,
    }

    impl<B, O> WriteTreeHeaderNext<B, O> {
        pub fn tree_header_written(self, written: O, context: &mut Context<B, O>) -> Result<Continue<B, O>, Error<O>>
        where O: Clone + PartialEq,
        {
            if written != context.tree_header_size {
                return Err(Error::WriteTreeHeaderSizeMismatch {
                    written,
                    expected: context.tree_header_size.clone(),
                });
            }
            context.tree_header_written = true;
            Ok(Continue {
                fold_action: FoldAction::Idle(self.fold_next),
                next: self.script,
            })
        }

    }


    pub struct WriteLevelHeader<B, O> {
        pub level_index: usize,
        pub level_offset: O,
        pub next: WriteLevelHeaderNext<B, O>,
    }

    pub struct WriteLevelHeaderNext<B, O> {
        level_seed: LevelSeed<B, O>,
        script: Script,
        fold_next: fold::VisitLevelNext,
    }

    impl<B, O> WriteLevelHeaderNext<B, O> {
        pub fn level_header_written(self, written: O, context: &mut Context<B, O>) -> Result<Continue<B, O>, Error<O>>
        where O: PartialEq
        {
            if written != self.level_seed.level_header_size {
                return Err(Error::WriteLevelHeaderSizeMismatch {
                    written,
                    expected: self.level_seed.level_header_size,
                });
            }
            let fold_continue = self
                .fold_next.level_ready(self.level_seed, context.fold_ctx())
                .map_err(Error::WriteLevelHeader)?;
            Ok(Continue {
                fold_action: FoldAction::Step(fold_continue),
                next: self.script,
            })
        }
    }


    pub struct WriteBlockHeader<B, O> {
        pub level_index: usize,
        pub block_index: usize,
        pub block_offset: O,
        pub items_count: usize,
        pub next: WriteBlockHeaderNext<B, O>,
    }

    pub struct WriteBlockHeaderNext<B, O> {
        level_seed: LevelSeed<B, O>,
        script: Script,
        fold_next: fold::VisitBlockStartNext,
    }

    impl<B, O> WriteBlockHeaderNext<B, O> {
        pub fn block_header_written(mut self, block_meta: B, written: O, context: &mut Context<B, O>) -> Result<Continue<B, O>, Error<O>>
        where O: Add<Output = O>,
        {
            self.level_seed.cursor = self.level_seed.cursor + written;
            self.level_seed.active_block = Some(block_meta);
            let fold_continue = self
                .fold_next.block_ready(self.level_seed, context.fold_ctx())
                .map_err(Error::WriteBlockHeader)?;
            Ok(Continue {
                fold_action: FoldAction::Step(fold_continue),
                next: self.script,
            })
        }
    }


    pub struct WriteItem<B, O> {
        pub level_index: usize,
        pub block_index: usize,
        pub block_item_index: usize,
        pub block_meta: B,
        pub child_block_offset: Option<O>,
        pub next: WriteItemNext<B, O>,
    }

    pub struct WriteItemNext<B, O> {
        level_seed: LevelSeed<B, O>,
        script: Script,
        fold_next: fold::VisitItemNext,
    }

    impl<B, O> WriteItemNext<B, O> {
        pub fn item_written(mut self, block_meta: B, written: O, context: &mut Context<B, O>) -> Result<Continue<B, O>, Error<O>>
        where O: Add<Output = O>,
        {
            self.level_seed.cursor = self.level_seed.cursor + written;
            self.level_seed.active_block = Some(block_meta);
            let fold_continue = self
                .fold_next.item_ready(self.level_seed, context.fold_ctx())
                .map_err(Error::WriteItem)?;
            context.recent_block_offset = None;
            Ok(Continue {
                fold_action: FoldAction::Step(fold_continue),
                next: self.script,
            })
        }
    }


    pub struct FlushBlock<B, O> {
        pub level_index: usize,
        pub block_index: usize,
        pub block_meta: B,
        pub block_start_offset: O,
        pub block_end_offset: O,
        pub next: FlushBlockNext<B, O>,
    }

    pub struct FlushBlockNext<B, O> {
        level_seed: LevelSeed<B, O>,
        script: Script,
        fold_next: fold::VisitBlockFinishNext,
    }

    impl<B, O> FlushBlockNext<B, O> {
        pub fn block_flushed(mut self, actual_block_size: O, context: &mut Context<B, O>) -> Result<Continue<B, O>, Error<O>>
        where O: Clone + Add<Output = O>,
        {
            context.recent_block_offset =
                Some(self.level_seed.block_offset.clone());
            self.level_seed.block_offset = self.level_seed.block_offset + actual_block_size;
            let fold_continue = self
                .fold_next.block_flushed(self.level_seed, context.fold_ctx())
                .map_err(Error::FlushBlock)?;
            Ok(Continue {
                fold_action: FoldAction::Step(fold_continue),
                next: self.script,
            })
        }
    }


    impl<B, O> Continue<B, O> {
        pub fn step_rec(self, write_context: &mut Context<B, O>) -> Result<Instruction<B, O>, Error<O>>
        where O: Clone + Add<Output = O>,
        {
            match self {
                Continue { fold_action: FoldAction::Idle(fold_op), next: write_next, } =>
                    write_next.step(write_context, fold_op),
                Continue { fold_action: FoldAction::Step(fold_continue), next: write_next, } => {
                    let fold_op = fold_continue
                        .step_rec(write_context.fold_ctx())
                        .map_err(Error::StepRec)?;
                    write_next.step(write_context, fold_op)
                },
            }
        }
    }
}
