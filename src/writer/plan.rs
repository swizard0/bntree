use super::super::sketch;

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
    BlockStart,
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
    level_base: usize,
}

struct LevelCursor {
    level_index: usize,
    block_index: usize,
    block_cursor: BlockCursor,
    items_remain: usize,
}

enum BlockCursor {
    Start,
    Write { index: usize, },
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
                    block_cursor: BlockCursor::Start,
                    items_remain: level.items_count,
                })
                .collect(),
            block_size: sketch.block_size(),
            level_curr: 0,
            level_base: 0,
        }
    }
}

impl Script {
    pub fn boot() -> Continue {
        Continue { next: Script(()), }
    }

    pub fn step(self, context: &mut Context) -> Instruction {
        if context.level_curr < context.cursors.len() {
            let cursor = &mut context.cursors[context.level_curr];
            if cursor.items_remain == 0 {
                context.level_base += 1;
                context.level_curr = context.level_base;
                return Instruction::Perform(Perform {
                    op: Op::BlockFinish,
                    level_index: cursor.level_index,
                    block_index: cursor.block_index,
                    next: Continue { next: self, },
                });
            }
            match cursor.block_cursor {
                BlockCursor::Start => {
                    cursor.block_cursor = BlockCursor::Write { index: 0, };
                    return Instruction::Perform(Perform {
                        op: Op::BlockStart,
                        level_index: cursor.level_index,
                        block_index: cursor.block_index,
                        next: Continue { next: self, },
                    });
                },
                BlockCursor::Write { index, } if index < context.block_size => {
                    cursor.block_cursor = BlockCursor::Write { index: index + 1, };
                    cursor.items_remain -= 1;
                    context.level_curr = context.level_base;
                    return Instruction::Perform(Perform {
                        op: Op::BlockItem { index, },
                        level_index: cursor.level_index,
                        block_index: cursor.block_index,
                        next: Continue { next: self, },
                    });
                },
                BlockCursor::Write { .. } => {
                    cursor.block_cursor = BlockCursor::Start;
                    let block_index = cursor.block_index;
                    cursor.block_index += 1;
                    context.level_curr += 1;
                    return Instruction::Perform(Perform {
                        op: Op::BlockFinish,
                        level_index: cursor.level_index,
                        block_index,
                        next: Continue { next: self, },
                    });
                },
            }
        } else {
            Instruction::Done
        }
    }
}
