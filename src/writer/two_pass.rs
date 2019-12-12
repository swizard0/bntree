struct TreeCoords<O> {
    offset: O,
    header_size: O,
}

pub mod markup {
    use std::ops::Add;

    use super::{
        TreeCoords,
        super::fold,
    };

    pub enum Instruction<B, O> {
        Op(Op<B, O>),
        Done,
    }

    pub enum Op<B, O> {
        InitialLevelSize(InitialLevelSize),
        AllocBlock(AllocBlock<B, O>),
        WriteItem(WriteItem<B, O>),
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
        NoActiveBlockWhileWriteItem,
    }

    pub struct Context<O> {
        tree_coords: TreeCoords<O>,
        child_pending: bool,
    }

    impl<O> Context<O> {
        pub fn new(tree_offset: O, tree_header_size: O) -> Context<O> {
            Context {
                tree_coords: TreeCoords {
                    offset: tree_offset,
                    header_size: tree_header_size,
                },
                child_pending: false,
            }
        }
    }

    pub struct Script(());

    impl Script {
        pub fn new() -> Script {
            Script(())
        }

        pub fn step<'s, B, O>(self, context: &mut Context<O>, op: fold::Instruction<LevelSeed<B, O>>) -> Result<Instruction<B, O>, Error> {
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
                    level_index, level_seed, block_index, next,
                })) =>
                    unimplemented!(),
                //                     Instruction::FinishMarkupBlock(FinishMarkupBlock {
                //                         level,
                //                         block,
                //                         next: FinishMarkupBlockNext {
                //                             fold_levels_next: next,
                //                             tree_coords: self.coords,
                //                             level_seed,
                //                             _marker: PhantomData,
                //                         },
                //                     }),
                fold::Instruction::Done => {
                    unimplemented!()
                    //                     let sketch = done.sketch();
                    //                     let mut iter = done.levels_iter();
                    //                     if let Some((level, markup)) = iter.next() {
                    //                         let markup_car = LevelMarkup { level, markup, };
                    //                         let mut markup_cdr = Vec::new();
                    //                         let mut tree_total_size = self.coords.header_size.clone() +
                    //                             markup_car.markup.level_size.clone();
                    //                         for (level, markup) in iter {
                    //                             tree_total_size = tree_total_size +
                    //                                 markup.level_size.clone();
                    //                             markup_cdr.push(LevelMarkup { level, markup, });
                    //                         }
                    //                         Instruction::WriteTreeHeader(WriteTreeHeader {
                    //                             offset: self.coords.offset.clone(),
                    //                             tree_total_size,
                    //                             next: WriteTreeHeaderNext {
                    //                                 sketch,
                    //                                 levels_markup: LevelsMarkup {
                    //                                     markup_car,
                    //                                     markup_cdr,
                    //                                     _marker: PhantomData,
                    //                                 },
                    //                                 tree_coords: self.coords,
                    //                                 _marker: PhantomData,
                    //                             },
                    //                         })
                    //                     } else {
                    //                         Instruction::Done
                    //                     }
                },
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
        pub fn level_header_size<'s, B, O>(
            self,
            level_header_size: O,
            context: &mut Context<O>,
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
        pub fn block_ready<'s>(
            self,
            block: B,
            context: &mut Context<O>,
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
        pub fn item_written<'s>(
            self,
            block: B,
            context: &mut Context<O>,
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
            Ok(Continue {
                fold_action: FoldAction::Step(fold_continue),
                next: self.script,
            })
        }
    }

}

//         match (self.inner, arg) {
//             (Fsm::Init { tree_offset, tree_header_size, }, StepArg::Markup { op, sketch, }) =>
//                 BusyMarkup {
//                     child_pending: false,
//                     coords: TreeCoords {
//                         offset: tree_offset,
//                         header_size: tree_header_size,
//                     },
//                 }.step(op, sketch),
//             (Fsm::Init { .. }, StepArg::Write { .. }) =>
//                 Err(Error::InvalidWriteArgForMarkupMode),
//             (Fsm::BusyMarkup(busy), StepArg::Markup { op, sketch, }) =>
//                 busy.step(op, sketch),
//             (Fsm::BusyMarkup(..), StepArg::Write { .. }) =>
//                 Err(Error::InvalidWriteArgForMarkupMode),
//             (Fsm::BusyWrite(busy), StepArg::Write { op, sketch, }) =>
//                 busy.step(op, sketch),
//             (Fsm::BusyWrite(..), StepArg::Markup { .. }) =>
//                 Err(Error::InvalidMarkupArgForWriteMode),
//         }


// struct BusyMarkup<O> {
//     child_pending: bool,
//     coords: TreeCoords<O>,
// }

// impl<O> BusyMarkup<O> {
//     fn step<'s, BA, BW>(
//         self,
//         op: fold::Instruction<MarkupLevelSeed<BA, O>>,
//         sketch: &'s sketch::Tree,
//     )
//         -> Result<Instruction<BA, BW, O>, Error>
//     {
//         match op {
//             fold::Instruction::Perform(fold::Perform { op: fold::Op::VisitLevel(fold::VisitLevel { level_index, .. }), .. }) =>
//                 Ok(Instruction::PassMarkup(PassMarkup {
//                     op: OpMarkup::InitialLevelSize(InitialLevelSize {
//                         level_index,
//                         next: InitialLevelSizeNext {
//                             level_index,
//                             busy: self,
//                         },
//                     }),
//                     next_fold: op,
//                 })),
//             fold::Instruction::Perform(
//                 fold::Perform { op: fold::Op::VisitBlockStart(fold::VisitBlockStart { level_index, block_index, .. }), .. }
//             ) =>
//                 Ok(Instruction::PassMarkup(PassMarkup {
//                     op: OpMarkup::AllocMarkupBlock(AllocMarkupBlock {
//                         level_index,
//                         block_index,
//                         next: AllocMarkupBlockNext {
//                             level_index,
//                             block_index,
//                             busy: self,
//                         },
//                     }),
//                     next_fold: op,
//                 })),
//             fold::Instruction::Perform(fold::Perform {
//                 op: fold::Op::VisitItem(fold::VisitItem { level_index, level_seed, next, .. }),
//                 next_plan,
//             }) =>
//                 unimplemented!(),
// //                     Instruction::WriteMarkupItem(WriteMarkupItem {
// //                         level,
// //                         block,
// //                         child_pending: markup.child_pending,
// //                         next: WriteMarkupItemNext {
// //                             fold_levels_next: next,
// //                             tree_coords: self.coords,
// //                             level_seed,
// //                             _marker: PhantomData,
// //                         },
// //                     }),
//             fold::Instruction::Perform(fold::Perform {
//                 op: fold::Op::VisitBlockFinish(fold::VisitBlockFinish { level_index, level_seed, next, .. }),
//                 next_plan,
//             }) =>
//                 unimplemented!(),
// //                     Instruction::FinishMarkupBlock(FinishMarkupBlock {
// //                         level,
// //                         block,
// //                         next: FinishMarkupBlockNext {
// //                             fold_levels_next: next,
// //                             tree_coords: self.coords,
// //                             level_seed,
// //                             _marker: PhantomData,
// //                         },
// //                     }),
//             fold::Instruction::Done(done) => {
//                 unimplemented!()
// //                     let sketch = done.sketch();
// //                     let mut iter = done.levels_iter();
// //                     if let Some((level, markup)) = iter.next() {
// //                         let markup_car = LevelMarkup { level, markup, };
// //                         let mut markup_cdr = Vec::new();
// //                         let mut tree_total_size = self.coords.header_size.clone() +
// //                             markup_car.markup.level_size.clone();
// //                         for (level, markup) in iter {
// //                             tree_total_size = tree_total_size +
// //                                 markup.level_size.clone();
// //                             markup_cdr.push(LevelMarkup { level, markup, });
// //                         }
// //                         Instruction::WriteTreeHeader(WriteTreeHeader {
// //                             offset: self.coords.offset.clone(),
// //                             tree_total_size,
// //                             next: WriteTreeHeaderNext {
// //                                 sketch,
// //                                 levels_markup: LevelsMarkup {
// //                                     markup_car,
// //                                     markup_cdr,
// //                                     _marker: PhantomData,
// //                                 },
// //                                 tree_coords: self.coords,
// //                                 _marker: PhantomData,
// //                             },
// //                         })
// //                     } else {
// //                         Instruction::Done
// //                     }
//             },
//         }
//     }
// }


// struct BusyWrite<O> {
//     _x: O,
// }

// impl<O> BusyWrite<O> {
//     fn step<'s, BA, BW>(self, op: fold::Instruction<BW>, sketch: &'s sketch::Tree) -> Result<Instruction<BA, BW, O>, Error> {
//         unimplemented!()
//     }
// }

// // pub fn write_blocks<'s, B1, B2, O>(
// //     sketch: &'s sketch::Tree,
// //     tree_offset: O,
// //     tree_header_size: O,
// // )
// //     -> WriteBlocks<'s, B1, B2, O>
// // {
// //     WriteBlocks {
// //         inner: Pass::Markup(Markup {
// //             fold_levels: fold::fold_levels(sketch),
// //             child_pending: false,
// //             _marker: PhantomData,
// //         }),
// //         coords: TreeCoords {
// //             offset: tree_offset,
// //             header_size: tree_header_size,
// //         },
// //     }
// // }

// // pub struct WriteBlocks<'s, B1, B2, O> {
// //     inner: Pass<'s, B1, B2, O>,
// //     coords: TreeCoords<O>,
// // }

// // enum Pass<'s, B1, B2, O> {
// //     Markup(Markup<'s, B1, B2, O>),
// //     Write(Write<'s, B1, B2, O>),
// // }

// // impl<'s, B1, B2, O> WriteBlocks<'s, B1, B2, O> where O: Add<Output = O> + Clone {
// //     pub fn next(self) -> Instruction<'s, B1, B2, O> {
// //         match self.inner {
// //             Pass::Markup(markup) => match markup.fold_levels.next() {
// //                 fold::Instruction::VisitLevel(fold::VisitLevel { level, next }) =>
// //                     Instruction::InitialLevelSize(InitialLevelSize {
// //                         level,
// //                         next: InitialLevelSizeNext {
// //                             fold_levels_next: next,
// //                             tree_coords: self.coords,
// //                             child_pending: markup.child_pending,
// //                             _marker: PhantomData,
// //                         },
// //                     }),
// //                 fold::Instruction::VisitBlockStart(fold::VisitBlockStart { level, level_seed, next, .. }) =>
// //                     Instruction::AllocMarkupBlock(AllocMarkupBlock {
// //                         level,
// //                         next: AllocMarkupBlockNext {
// //                             fold_levels_next: next,
// //                             tree_coords: self.coords,
// //                             child_pending: markup.child_pending,
// //                             level_seed,
// //                             _marker: PhantomData,
// //                         },
// //                     }),
// //                 fold::Instruction::VisitItem(fold::VisitItem { level, level_seed, block, next, .. }) =>
// //                     Instruction::WriteMarkupItem(WriteMarkupItem {
// //                         level,
// //                         block,
// //                         child_pending: markup.child_pending,
// //                         next: WriteMarkupItemNext {
// //                             fold_levels_next: next,
// //                             tree_coords: self.coords,
// //                             level_seed,
// //                             _marker: PhantomData,
// //                         },
// //                     }),
// //                 fold::Instruction::VisitBlockFinish(fold::VisitBlockFinish { level, level_seed, block, next, .. }) =>
// //                     Instruction::FinishMarkupBlock(FinishMarkupBlock {
// //                         level,
// //                         block,
// //                         next: FinishMarkupBlockNext {
// //                             fold_levels_next: next,
// //                             tree_coords: self.coords,
// //                             level_seed,
// //                             _marker: PhantomData,
// //                         },
// //                     }),
// //                 fold::Instruction::Done(done) => {
// //                     let sketch = done.sketch();
// //                     let mut iter = done.levels_iter();
// //                     if let Some((level, markup)) = iter.next() {
// //                         let markup_car = LevelMarkup { level, markup, };
// //                         let mut markup_cdr = Vec::new();
// //                         let mut tree_total_size = self.coords.header_size.clone() +
// //                             markup_car.markup.level_size.clone();
// //                         for (level, markup) in iter {
// //                             tree_total_size = tree_total_size +
// //                                 markup.level_size.clone();
// //                             markup_cdr.push(LevelMarkup { level, markup, });
// //                         }
// //                         Instruction::WriteTreeHeader(WriteTreeHeader {
// //                             offset: self.coords.offset.clone(),
// //                             tree_total_size,
// //                             next: WriteTreeHeaderNext {
// //                                 sketch,
// //                                 levels_markup: LevelsMarkup {
// //                                     markup_car,
// //                                     markup_cdr,
// //                                     _marker: PhantomData,
// //                                 },
// //                                 tree_coords: self.coords,
// //                                 _marker: PhantomData,
// //                             },
// //                         })
// //                     } else {
// //                         Instruction::Done
// //                     }
// //                 },
// //             },
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
