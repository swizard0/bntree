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
        Done(Done),
    }

    pub enum Op<B, O> {
        InitialLevelSize(InitialLevelSize),
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
    }

    pub struct Context {
        child_pending: bool,
    }

    impl Context {
        pub fn new() -> Context {
            Context {
                child_pending: false,
            }
        }
    }

    pub struct Script(());

    impl Script {
        pub fn new() -> Script {
            Script(())
        }

        pub fn step<B, O>(self, context: &mut Context, op: fold::Instruction<LevelSeed<B, O>>) -> Result<Instruction<B, O>, Error> {
            match op {
                fold::Instruction::Op(fold::Op::VisitLevel(fold::VisitLevel { level_index, next, })) =>
                    Ok(Instruction::Op(Op::InitialLevelSize(InitialLevelSize {
                        level_index,
                        next: InitialLevelSizeNext {
                            script: self,
                            fold_next: next,
                        },
                    }))),
                fold::Instruction::Op(fold::Op::VisitBlockStart(fold::VisitBlockStart { level_index, level_seed, block_index, next, })) =>
                    Ok(Instruction::Op(Op::AllocBlock(AllocBlock {
                        level_index,
                        block_index,
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
                    Ok(Instruction::Done(Done(()))),
            }
        }
    }

    pub struct LevelSeed<B, O> {
        level_header_size: O,
        level_size: O,
        active_block: Option<B>,
    }


    pub struct InitialLevelSize {
        pub level_index: usize,
        pub next: InitialLevelSizeNext,
    }

    pub struct InitialLevelSizeNext {
        script: Script,
        fold_next: fold::VisitLevelNext,
    }

    impl InitialLevelSizeNext {
        pub fn level_header_size<B, O>(
            self,
            level_header_size: O,
            fold_ctx: &mut fold::Context<LevelSeed<B, O>>,
        )
            -> Result<Continue<B, O>, Error>
        where O: Clone
        {
            let fold_continue =
                self.fold_next.level_ready(
                    LevelSeed {
                        level_size: level_header_size.clone(),
                        level_header_size,
                        active_block: None,
                    },
                    fold_ctx,
                ).map_err(Error::LevelHeaderSize)?;
            Ok(Continue {
                fold_action: FoldAction::Step(fold_continue),
                next: self.script,
            })
        }
    }


    pub struct AllocBlock<B, O> {
        pub level_index: usize,
        pub block_index: usize,
        pub next: AllocBlockNext<B, O>,
    }

    pub struct AllocBlockNext<B, O> {
        level_seed: LevelSeed<B, O>,
        script: Script,
        fold_next: fold::VisitBlockStartNext,
    }

    impl<B, O> AllocBlockNext<B, O> {
        pub fn block_ready(
            self,
            block: B,
            fold_ctx: &mut fold::Context<LevelSeed<B, O>>,
        )
            -> Result<Continue<B, O>, Error>
        where O: Add<Output = O>
        {
            let fold_continue =
                self.fold_next.block_ready(
                    LevelSeed {
                        active_block: Some(block),
                        ..self.level_seed
                    },
                    fold_ctx,
                ).map_err(Error::LevelHeaderSize)?;
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
        pub fn item_written(
            self,
            block: B,
            context: &mut Context,
            fold_ctx: &mut fold::Context<LevelSeed<B, O>>,
        )
            -> Result<Continue<B, O>, Error>
        {
            let fold_continue =
                self.fold_next.item_ready(
                    LevelSeed {
                        active_block: Some(block),
                        ..self.level_seed
                    },
                    fold_ctx,
                ).map_err(Error::WriteItem)?;
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
        pub fn block_finished(
            self,
            block_size: O,
            context: &mut Context,
            fold_ctx: &mut fold::Context<LevelSeed<B, O>>,
        )
            -> Result<Continue<B, O>, Error>
        where O: Add<Output = O>,
        {
            let fold_continue =
                self.fold_next.block_flushed(
                    LevelSeed {
                        active_block: None,
                        level_size: self.level_seed.level_size + block_size,
                        ..self.level_seed
                    },
                    fold_ctx,
                ).map_err(Error::FinishBlock)?;
            context.child_pending = true;
            Ok(Continue {
                fold_action: FoldAction::Step(fold_continue),
                next: self.script,
            })
        }
    }

    pub struct Done(());

    impl Done {
        pub fn finish<B, O>(self, fold_ctx: fold::Context<LevelSeed<B, O>>) -> impl Iterator<Item = super::LevelCoords<O>> {
            fold_ctx
                .levels_iter()
                .map(|(index, LevelSeed { level_header_size, level_size, .. })| {
                    super::LevelCoords {
                        index,
                        header_size: level_header_size,
                        total_size: level_size,
                    }
                })
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
    }

    pub struct Context<O> {
        tree_offset: O,
        tree_header_size: O,
        tree_header_written: bool,
        recent_block_offset: Option<O>,
        levels_car: LevelCoords<O>,
        levels_cdr: Vec<LevelCoords<O>>,
    }

    impl<O> Context<O> {
        pub fn new<I>(tree_offset: O, tree_header_size: O, mut levels_coords: I) -> Result<Context<O>, Error<O>>
        where I: Iterator<Item = LevelCoords<O>>
        {
            if let Some(levels_car) = levels_coords.next() {
                Ok(Context {
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
    }

    pub struct Script(());

    impl Script {
        pub fn step<B, O>(self, context: &mut Context<O>, op: fold::Instruction<LevelSeed<B, O>>) -> Result<Instruction<B, O>, Error<O>>
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
                fold::Instruction::Op(fold::Op::VisitBlockStart(fold::VisitBlockStart { level_index, level_seed, block_index, next, })) =>
                    unimplemented!(),
                    // Ok(Instruction::Op(Op::AllocBlock(AllocBlock {
                    //     level_index,
                    //     block_index,
                    //     next: AllocBlockNext {
                    //         level_seed,
                    //         script: self,
                    //         fold_next: next,
                    //     },
                    // }))),
                fold::Instruction::Op(fold::Op::VisitItem(fold::VisitItem {
                    level_index, mut level_seed, block_index, block_item_index, next,
                })) =>
                    unimplemented!(),
                    // if let Some(block) = level_seed.active_block.take() {
                    //     Ok(Instruction::Op(Op::WriteItem(WriteItem {
                    //         level_index,
                    //         block_index,
                    //         block_item_index,
                    //         block,
                    //         child_pending: context.child_pending,
                    //         next: WriteItemNext {
                    //             level_seed,
                    //             script: self,
                    //             fold_next: next,
                    //         },
                    //     })))
                    // } else {
                    //     Err(Error::NoActiveBlockWhileWriteItem)
                    // },
                fold::Instruction::Op(fold::Op::VisitBlockFinish(fold::VisitBlockFinish {
                    level_index, mut level_seed, block_index, next,
                })) =>
                    unimplemented!(),
                    // if let Some(block) = level_seed.active_block.take() {
                    //     Ok(Instruction::Op(Op::FinishBlock(FinishBlock {
                    //         level_index,
                    //         block_index,
                    //         block,
                    //         next: FinishBlockNext {
                    //             level_seed,
                    //             script: self,
                    //             fold_next: next,
                    //         },
                    //     })))
                    // } else {
                    //     Err(Error::NoActiveBlockWhileBlockFinish)
                    // },
                fold::Instruction::Done =>
                    unimplemented!(),
                    // Ok(Instruction::Done(Done(()))),
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
        pub fn tree_header_written(
            self,
            written: O,
            context: &mut Context<O>,
        )
            -> Result<Continue<B, O>, Error<O>>
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
        pub fn level_header_written(
            self,
            written: O,
            context: &mut Context<O>,
            fold_ctx: &mut fold::Context<LevelSeed<B, O>>,
        )
            -> Result<Continue<B, O>, Error<O>>
        where O: PartialEq
        {
            if written != self.level_seed.level_header_size {
                return Err(Error::WriteLevelHeaderSizeMismatch {
                    written,
                    expected: self.level_seed.level_header_size,
                });
            }
            let fold_continue = self
                .fold_next.level_ready(self.level_seed, fold_ctx)
                .map_err(Error::WriteLevelHeader)?;
            Ok(Continue {
                fold_action: FoldAction::Step(fold_continue),
                next: self.script,
            })
        }
    }

}

// // impl<'s, B1, B2, O> WriteBlocks<'s, B1, B2, O> where O: Add<Output = O> + Clone {
// //     pub fn next(self) -> Instruction<'s, B1, B2, O> {
// //         match self.inner {
// //             Pass::Write(mut write) => match write.fold_levels.next() {
// //                 fold::Instruction::VisitLevel(fold::VisitLevel { level, next, }) => {
// //                     let mut level_offset = self.coords.header_size.clone();
// //                     let mut level_markup = &write.levels_markup.markup_car;
// //                     let mut level_header_size =
// //                         level_markup.markup.level_header_size.clone();
// //                     let mut iter = write.levels_markup.markup_cdr.iter();
// //                     loop {
// //                         if level_markup.level.index >= level.index {
// //                             break;
// //                         }
// //                         level_offset = level_offset +
// //                             level_markup.markup.level_size.clone();
// //                         level_header_size =
// //                             level_markup.markup.level_header_size.clone();
// //                         if let Some(markup) = iter.next() {
// //                             level_markup = markup;
// //                         } else {
// //                             break;
// //                         }
// //                     }
// //                     let block_offset =
// //                         level_offset.clone() + level_header_size.clone();
// //                     Instruction::WriteLevelHeader(WriteLevelHeader {
// //                         level,
// //                         level_offset,
// //                         next: WriteLevelHeaderNext {
// //                             fold_levels_next: next,
// //                             levels_markup: write.levels_markup,
// //                             recent_block_offset: write.recent_block_offset,
// //                             tree_coords: self.coords,
// //                             level_seed: WriteLevelSeed {
// //                                 level_header_size,
// //                                 cursor: block_offset.clone(),
// //                                 block_offset,
// //                             },
// //                         },
// //                     })
// //                 },
// //                 fold::Instruction::VisitBlockStart(fold::VisitBlockStart { level, level_seed, next, .. }) =>
// //                     Instruction::WriteBlockHeader(WriteBlockHeader {
// //                         level,
// //                         block_offset: level_seed.block_offset.clone(),
// //                         next: WriteBlockHeaderNext {
// //                             fold_levels_next: next,
// //                             levels_markup: write.levels_markup,
// //                             recent_block_offset: write.recent_block_offset,
// //                             tree_coords: self.coords,
// //                             level_seed,
// //                         },
// //                     }),
// //                 fold::Instruction::VisitItem(fold::VisitItem { level, level_seed, block, next, .. }) =>
// //                     Instruction::WriteBlockItem(WriteBlockItem {
// //                         level,
// //                         block_meta: block,
// //                         item_offset: level_seed.cursor.clone(),
// //                         child_block_offset: write.recent_block_offset.take(),
// //                         next: WriteBlockItemNext {
// //                             fold_levels_next: next,
// //                             levels_markup: write.levels_markup,
// //                             tree_coords: self.coords,
// //                             level_seed,
// //                         },
// //                     }),
// //                 fold::Instruction::VisitBlockFinish(fold::VisitBlockFinish { level, level_seed, block, next, .. }) =>
// //                     Instruction::FlushBlock(FlushBlock {
// //                         level,
// //                         block_meta: block,
// //                         block_start_offset: level_seed.block_offset.clone(),
// //                         block_end_offset: level_seed.cursor.clone(),
// //                         next: FlushBlockNext {
// //                             fold_levels_next: next,
// //                             levels_markup: write.levels_markup,
// //                             tree_coords: self.coords,
// //                             level_seed,
// //                         },
// //                     }),
// //                 fold::Instruction::Done(..) =>
// //                     Instruction::Done,
// //             },
// //         }
// //     }
// // }

// // // Instruction
// // pub enum Instruction<'s, B1, B2, O> {
// //     InitialLevelSize(InitialLevelSize<'s, B1, B2, O>),
// //     AllocMarkupBlock(AllocMarkupBlock<'s, B1, B2, O>),
// //     WriteMarkupItem(WriteMarkupItem<'s, B1, B2, O>),
// //     FinishMarkupBlock(FinishMarkupBlock<'s, B1, B2, O>),
// //     WriteTreeHeader(WriteTreeHeader<'s, B1, B2, O>),
// //     WriteLevelHeader(WriteLevelHeader<'s, B1, B2, O>),
// //     WriteBlockHeader(WriteBlockHeader<'s, B1, B2, O>),
// //     WriteBlockItem(WriteBlockItem<'s, B1, B2, O>),
// //     FlushBlock(FlushBlock<'s, B1, B2, O>),
// //     Done,
// // }

// // // Markup
// // struct Markup<'s, B1, B2, O> {
// //     fold_levels: fold::FoldLevels<'s, B1, MarkupLevelSeed<O>>,
// //     child_pending: bool,
// //     _marker: PhantomData<B2>,
// // }

// // struct MarkupLevelSeed<O> {
// //     level_header_size: O,
// //     level_size: O,
// // }

// // // InitialLevelSize
// // pub struct InitialLevelSize<'s, B1, B2, O> {
// //     pub level: &'s sketch::Level,
// //     pub next: InitialLevelSizeNext<'s, B1, B2, O>,
// // }

// // pub struct InitialLevelSizeNext<'s, B1, B2, O> {
// //     fold_levels_next: fold::VisitLevelNext<'s, B1, MarkupLevelSeed<O>>,
// //     tree_coords: TreeCoords<O>,
// //     child_pending: bool,
// //     _marker: PhantomData<B2>,
// // }

// // impl<'s, B1, B2, O> InitialLevelSizeNext<'s, B1, B2, O> where O: Clone {
// //     pub fn level_header_size(self, level_header_size: O) -> AllocMarkupBlock<'s, B1, B2, O> {
// //         let fold::VisitBlockStart { level, level_seed, next: fold_levels_next, .. } =
// //             self.fold_levels_next.level_ready(MarkupLevelSeed {
// //                 level_size: level_header_size.clone(),
// //                 level_header_size,
// //             });
// //         AllocMarkupBlock {
// //             level,
// //             next: AllocMarkupBlockNext {
// //                 fold_levels_next,
// //                 tree_coords: self.tree_coords,
// //                 child_pending: self.child_pending,
// //                 level_seed,
// //                 _marker: PhantomData,
// //             },
// //         }
// //     }
// // }

// // // AllocMarkupBlock
// // pub struct AllocMarkupBlock<'s, B1, B2, O> {
// //     pub level: &'s sketch::Level,
// //     pub next: AllocMarkupBlockNext<'s, B1, B2, O>,
// // }

// // pub struct AllocMarkupBlockNext<'s, B1, B2, O> {
// //     fold_levels_next: fold::VisitBlockStartNext<'s, B1, MarkupLevelSeed<O>>,
// //     tree_coords: TreeCoords<O>,
// //     child_pending: bool,
// //     level_seed: MarkupLevelSeed<O>,
// //     _marker: PhantomData<B2>,
// // }

// // impl<'s, B1, B2, O> AllocMarkupBlockNext<'s, B1, B2, O> {
// //     pub fn level_size(&self) -> &O {
// //         &self.level_seed.level_size
// //     }

// //     pub fn block_ready(self, block: B1) -> WriteBlocks<'s, B1, B2, O> {
// //         WriteBlocks {
// //             inner: Pass::Markup(Markup {
// //                 fold_levels: self.fold_levels_next.block_ready(
// //                     block,
// //                     self.level_seed,
// //                 ),
// //                 child_pending: self.child_pending,
// //                 _marker: PhantomData,
// //             }),
// //             coords: self.tree_coords,
// //         }
// //     }
// // }

// // // WriteMarkupItem
// // pub struct WriteMarkupItem<'s, B1, B2, O> {
// //     pub level: &'s sketch::Level,
// //     pub block: B1,
// //     pub child_pending: bool,
// //     pub next: WriteMarkupItemNext<'s, B1, B2, O>,
// // }

// // pub struct WriteMarkupItemNext<'s, B1, B2, O> {
// //     fold_levels_next: fold::VisitItemNext<'s, B1, MarkupLevelSeed<O>>,
// //     tree_coords: TreeCoords<O>,
// //     level_seed: MarkupLevelSeed<O>,
// //     _marker: PhantomData<B2>,
// // }

// // impl<'s, B1, B2, O> WriteMarkupItemNext<'s, B1, B2, O> {
// //     pub fn level_size(&self) -> &O {
// //         &self.level_seed.level_size
// //     }

// //     pub fn item_written(self, block: B1) -> WriteBlocks<'s, B1, B2, O> {
// //         WriteBlocks {
// //             inner: Pass::Markup(Markup {
// //                 fold_levels: self.fold_levels_next.item_ready(
// //                     block,
// //                     self.level_seed,
// //                 ),
// //                 child_pending: false,
// //                 _marker: PhantomData,
// //             }),
// //             coords: self.tree_coords,
// //         }
// //     }
// // }

// // // FinishMarkupBlock
// // pub struct FinishMarkupBlock<'s, B1, B2, O> {
// //     pub level: &'s sketch::Level,
// //     pub block: B1,
// //     pub next: FinishMarkupBlockNext<'s, B1, B2, O>,
// // }

// // pub struct FinishMarkupBlockNext<'s, B1, B2, O> {
// //     fold_levels_next: fold::VisitBlockFinishNext<'s, B1, MarkupLevelSeed<O>>,
// //     tree_coords: TreeCoords<O>,
// //     level_seed: MarkupLevelSeed<O>,
// //     _marker: PhantomData<B2>,
// // }

// // impl<'s, B1, B2, O> FinishMarkupBlockNext<'s, B1, B2, O> where O: Add<Output = O> {
// //     pub fn level_size(&self) -> &O {
// //         &self.level_seed.level_size
// //     }

// //     pub fn block_finished(self, block_size: O) -> WriteBlocks<'s, B1, B2, O> {
// //         let level_size = self.level_seed.level_size + block_size;
// //         WriteBlocks {
// //             inner: Pass::Markup(Markup {
// //                 fold_levels: self.fold_levels_next.block_flushed(MarkupLevelSeed {
// //                     level_header_size: self.level_seed.level_header_size,
// //                     level_size,
// //                 }),
// //                 child_pending: true,
// //                 _marker: PhantomData,
// //             }),
// //             coords: self.tree_coords,
// //         }
// //     }
// // }

// // // Write
// // struct Write<'s, B1, B2, O> {
// //     fold_levels: fold::FoldLevels<'s, B2, WriteLevelSeed<O>>,
// //     levels_markup: LevelsMarkup<'s, B1, O>,
// //     recent_block_offset: Option<O>,
// // }

// // struct WriteLevelSeed<O> {
// //     level_header_size: O,
// //     block_offset: O,
// //     cursor: O,
// // }

// // struct LevelMarkup<'s, O> {
// //     level: &'s sketch::Level,
// //     markup: MarkupLevelSeed<O>,
// // }

// // struct LevelsMarkup<'s, B1, O> {
// //     markup_car: LevelMarkup<'s, O>,
// //     markup_cdr: Vec<LevelMarkup<'s, O>>,
// //     _marker: PhantomData<B1>,
// // }

// // // WriteTreeHeader
// // pub struct WriteTreeHeader<'s, B1, B2, O> {
// //     pub offset: O,
// //     pub tree_total_size: O,
// //     pub next: WriteTreeHeaderNext<'s, B1, B2, O>,
// // }

// // pub struct WriteTreeHeaderNext<'s, B1, B2, O> {
// //     sketch: &'s sketch::Tree,
// //     levels_markup: LevelsMarkup<'s, B1, O>,
// //     tree_coords: TreeCoords<O>,
// //     _marker: PhantomData<B2>,
// // }

// // impl<'s, B1, B2, O> WriteTreeHeaderNext<'s, B1, B2, O> {
// //     pub fn tree_header_written(self, written: O) -> WriteBlocks<'s, B1, B2, O> where O: PartialEq {
// //         assert!(written == self.tree_coords.header_size);
// //         WriteBlocks {
// //             inner: Pass::Write(Write {
// //                 fold_levels: fold::fold_levels(self.sketch),
// //                 levels_markup: self.levels_markup,
// //                 recent_block_offset: None,
// //             }),
// //             coords: self.tree_coords,
// //         }
// //     }
// // }

// // // WriteLevelHeader
// // pub struct WriteLevelHeader<'s, B1, B2, O> {
// //     pub level: &'s sketch::Level,
// //     pub level_offset: O,
// //     pub next: WriteLevelHeaderNext<'s, B1, B2, O>,
// // }

// // pub struct WriteLevelHeaderNext<'s, B1, B2, O> {
// //     fold_levels_next: fold::VisitLevelNext<'s, B2, WriteLevelSeed<O>>,
// //     levels_markup: LevelsMarkup<'s, B1, O>,
// //     recent_block_offset: Option<O>,
// //     tree_coords: TreeCoords<O>,
// //     level_seed: WriteLevelSeed<O>,
// // }

// // impl<'s, B1, B2, O> WriteLevelHeaderNext<'s, B1, B2, O> {
// //     pub fn level_header_written(self, written: O) -> WriteBlockHeader<'s, B1, B2, O> where O: Clone + PartialEq + Add<Output = O> {
// //         assert!(written == self.level_seed.level_header_size);
// //         let fold::VisitBlockStart { level, level_seed, next: fold_levels_next, .. } =
// //             self.fold_levels_next.level_ready(self.level_seed);
// //         WriteBlockHeader {
// //             level,
// //             block_offset: level_seed.block_offset.clone(),
// //             next: WriteBlockHeaderNext {
// //                 fold_levels_next,
// //                 levels_markup: self.levels_markup,
// //                 recent_block_offset: self.recent_block_offset,
// //                 tree_coords: self.tree_coords,
// //                 level_seed,
// //             },
// //         }
// //     }
// // }

// // // WriteBlockHeader
// // pub struct WriteBlockHeader<'s, B1, B2, O> {
// //     pub level: &'s sketch::Level,
// //     pub block_offset: O,
// //     pub next: WriteBlockHeaderNext<'s, B1, B2, O>,
// // }

// // pub struct WriteBlockHeaderNext<'s, B1, B2, O> {
// //     fold_levels_next: fold::VisitBlockStartNext<'s, B2, WriteLevelSeed<O>>,
// //     levels_markup: LevelsMarkup<'s, B1, O>,
// //     recent_block_offset: Option<O>,
// //     tree_coords: TreeCoords<O>,
// //     level_seed: WriteLevelSeed<O>,
// // }

// // impl<'s, B1, B2, O> WriteBlockHeaderNext<'s, B1, B2, O> {
// //     pub fn block_header_written(self, block_meta: B2, written: O) -> WriteBlocks<'s, B1, B2, O> where O: Add<Output = O> {
// //         WriteBlocks {
// //             inner: Pass::Write(Write {
// //                 fold_levels: self.fold_levels_next.block_ready(
// //                     block_meta,
// //                     WriteLevelSeed {
// //                         cursor: self.level_seed.cursor + written,
// //                         ..self.level_seed
// //                     },
// //                 ),
// //                 levels_markup: self.levels_markup,
// //                 recent_block_offset: self.recent_block_offset,
// //             }),
// //             coords: self.tree_coords,
// //         }
// //     }
// // }

// // // WriteBlockItem
// // pub struct WriteBlockItem<'s, B1, B2, O> {
// //     pub level: &'s sketch::Level,
// //     pub block_meta: B2,
// //     pub item_offset: O,
// //     pub child_block_offset: Option<O>,
// //     pub next: WriteBlockItemNext<'s, B1, B2, O>,
// // }

// // pub struct WriteBlockItemNext<'s, B1, B2, O> {
// //     fold_levels_next: fold::VisitItemNext<'s, B2, WriteLevelSeed<O>>,
// //     levels_markup: LevelsMarkup<'s, B1, O>,
// //     tree_coords: TreeCoords<O>,
// //     level_seed: WriteLevelSeed<O>,
// // }

// // impl<'s, B1, B2, O> WriteBlockItemNext<'s, B1, B2, O> {
// //     pub fn item_written(self, block_meta: B2, written: O) -> WriteBlocks<'s, B1, B2, O> where O: Add<Output = O> {
// //         WriteBlocks {
// //             inner: Pass::Write(Write {
// //                 fold_levels: self.fold_levels_next.item_ready(
// //                     block_meta,
// //                     WriteLevelSeed {
// //                         cursor: self.level_seed.cursor + written,
// //                         ..self.level_seed
// //                     },
// //                 ),
// //                 levels_markup: self.levels_markup,
// //                 recent_block_offset: None,
// //             }),
// //             coords: self.tree_coords,
// //         }
// //     }
// // }

// // // FlushBlock
// // pub struct FlushBlock<'s, B1, B2, O> {
// //     pub level: &'s sketch::Level,
// //     pub block_meta: B2,
// //     pub block_start_offset: O,
// //     pub block_end_offset: O,
// //     pub next: FlushBlockNext<'s, B1, B2, O>,
// // }

// // pub struct FlushBlockNext<'s, B1, B2, O> {
// //     fold_levels_next: fold::VisitBlockFinishNext<'s, B2, WriteLevelSeed<O>>,
// //     levels_markup: LevelsMarkup<'s, B1, O>,
// //     tree_coords: TreeCoords<O>,
// //     level_seed: WriteLevelSeed<O>,
// // }

// // impl<'s, B1, B2, O> FlushBlockNext<'s, B1, B2, O> {
// //     pub fn block_flushed(self, actual_block_size: O) -> WriteBlocks<'s, B1, B2, O> where O: Clone + Add<Output = O> {
// //         let recent_block_offset =
// //             Some(self.level_seed.block_offset.clone());
// //         WriteBlocks {
// //             inner: Pass::Write(Write {
// //                 fold_levels: self.fold_levels_next.block_flushed(
// //                     WriteLevelSeed {
// //                         block_offset: self.level_seed.block_offset.clone() + actual_block_size,
// //                         ..self.level_seed
// //                     },
// //                 ),
// //                 levels_markup: self.levels_markup,
// //                 recent_block_offset,
// //             }),
// //             coords: self.tree_coords,
// //         }
// //     }
// // }
