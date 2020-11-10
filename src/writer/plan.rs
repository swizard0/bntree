use crate::sketch;

pub enum Instruction {
    Perform(Perform),
    Done,
}

pub struct Perform {
    pub op: Op,
    pub level_index: usize,
    pub block_index: usize,
    pub next: Continue,
}

pub enum Op {
    BlockStart { items_count: usize, },
    BlockItem { index: usize, },
    BlockFinish,
}

pub struct Continue {
    pub next: Script,
}

pub struct Script(());

pub struct Context {
    cursors: Vec<LevelCursor>,
    block_size: usize,
    level_curr: usize,
}

struct LevelCursor {
    level_index: usize,
    block_index: usize,
    items_remain: usize,
    block_cursor: BlockCursor,
}

enum BlockCursor {
    Begin,
    Write { index: usize, },
    Commit,
}

impl Context {
    pub fn new(sketch: &sketch::Tree) -> Context {
        Context {
            cursors: sketch
                .levels()
                .iter()
                .rev()
                .map(|level| LevelCursor {
                    level_index: level.index,
                    block_index: 0,
                    items_remain: level.items_count,
                    block_cursor: BlockCursor::Begin,
                })
                .collect(),
            block_size: sketch.block_size(),
            level_curr: 0,
        }
    }
}

impl Script {
    pub fn boot() -> Continue {
        Continue { next: Script(()), }
    }

    pub fn step(self, context: &mut Context) -> Instruction {
        match context.cursors.get_mut(context.level_curr) {
            None =>
                Instruction::Done,
            Some(LevelCursor { block_cursor, items_remain, level_index, block_index, }) => {
                let items_count = if *items_remain < context.block_size {
                    *items_remain
                } else {
                    context.block_size
                };
                match *block_cursor {
                    BlockCursor::Begin => {
                        *block_cursor = BlockCursor::Write { index: 0, };
                        Instruction::Perform(Perform {
                            op: Op::BlockStart { items_count, },
                            level_index: *level_index,
                            block_index: *block_index,
                            next: Continue { next: self, },
                        })
                    },
                    BlockCursor::Write { index, } => {
                        let current_level_index = *level_index;
                        let current_block_index = *block_index;
                        let next_index = index + 1;
                        *items_remain -= 1;
                        if *items_remain == 0 || next_index == context.block_size {
                            *block_cursor = BlockCursor::Commit;
                        } else {
                            *block_cursor = BlockCursor::Write {
                                index: next_index,
                            };
                            // fall to the bottom
                            loop {
                                match context.level_curr.checked_sub(1).and_then(|index| context.cursors.get(index)) {
                                    Some(&LevelCursor { items_remain, .. }) if items_remain > 0 =>
                                        context.level_curr -= 1,
                                    _ =>
                                        break,
                                }
                            }
                        }
                        Instruction::Perform(Perform {
                            op: Op::BlockItem { index, },
                            level_index: current_level_index,
                            block_index: current_block_index,
                            next: Continue { next: self, },
                        })
                    },
                    BlockCursor::Commit => {
                        *block_cursor = BlockCursor::Begin;
                        let current_block_index = *block_index;
                        *block_index += 1;
                        context.level_curr += 1;
                        Instruction::Perform(Perform {
                            op: Op::BlockFinish,
                            level_index: *level_index,
                            block_index: current_block_index,
                            next: Continue { next: self, },
                        })
                    },
                }
            },
        }

        // if context.level_curr < context.cursors.len() {
        //     let cursor = &mut context.cursors[context.level_curr];
        //     if cursor.items_remain == 0 {
        //         context.level_base += 1;
        //         context.level_curr = context.level_base;
        //         return Instruction::Perform(Perform {
        //             op: Op::BlockFinish,
        //             level_index: cursor.level_index,
        //             block_index: cursor.block_index,
        //             next: Continue { next: self, },
        //         });
        //     }
        //     match cursor.block_cursor {
        //         BlockCursor::Start => {
        //             cursor.block_cursor = BlockCursor::Write { index: 0, };
        //             return Instruction::Perform(Perform {
        //                 op: Op::BlockStart {
        //                     items_count: if cursor.items_remain < context.block_size {
        //                         cursor.items_remain
        //                     } else {
        //                         context.block_size
        //                     },
        //                 },
        //                 level_index: cursor.level_index,
        //                 block_index: cursor.block_index,
        //                 next: Continue { next: self, },
        //             });
        //         },
        //         BlockCursor::Write { index, } if index < context.block_size => {
        //             cursor.block_cursor = BlockCursor::Write { index: index + 1, };
        //             cursor.items_remain -= 1;
        //             context.level_curr = context.level_base;
        //             return Instruction::Perform(Perform {
        //                 op: Op::BlockItem { index, },
        //                 level_index: cursor.level_index,
        //                 block_index: cursor.block_index,
        //                 next: Continue { next: self, },
        //             });
        //         },
        //         BlockCursor::Write { .. } => {
        //             cursor.block_cursor = BlockCursor::Start;
        //             let block_index = cursor.block_index;
        //             cursor.block_index += 1;
        //             context.level_curr += 1;
        //             return Instruction::Perform(Perform {
        //                 op: Op::BlockFinish,
        //                 level_index: cursor.level_index,
        //                 block_index,
        //                 next: Continue { next: self, },
        //             });
        //         },
        //     }
        // } else {
        //     Instruction::Done
        // }
    }
}
