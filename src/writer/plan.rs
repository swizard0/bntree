use super::super::sketch;

pub enum Instruction {
    Perform(Perform),
    Done,
}

pub struct Perform {
    pub op: Op,
    pub level_index: usize,
    pub block_index: usize,
    pub next: Script,
}

pub enum Op {
    BlockStart,
    BlockItem { index: usize, },
    BlockFinish,
}

pub struct Script {
    inner: Fsm,
}

enum Fsm {
    Init,
    Busy(Busy),
}

struct Busy {
    cursors: Vec<LevelCursor>,
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

impl Script {
    pub fn new() -> Script {
        Script { inner: Fsm::Init, }
    }

    pub fn step(self, sketch: &sketch::Tree) -> Instruction {
        match self.inner {
            Fsm::Init =>
                Busy {
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
                    level_curr: 0,
                    level_base: 0,
                },
            Fsm::Busy(busy) =>
                busy,
        }.step(sketch)
    }
}

impl Busy {
    fn step(mut self, sketch: &sketch::Tree) -> Instruction {
        if self.level_curr < self.cursors.len() {
            let cursor = &mut self.cursors[self.level_curr];
            if cursor.items_remain == 0 {
                return Instruction::Perform(Perform {
                    op: Op::BlockFinish,
                    level_index: cursor.level_index,
                    block_index: cursor.block_index,
                    next: Script {
                        inner: Fsm::Busy(Busy {
                            level_base: self.level_base + 1,
                            level_curr: self.level_base + 1,
                            ..self
                        }),
                    },
                });
            }
            match cursor.block_cursor {
                BlockCursor::Start => {
                    cursor.block_cursor = BlockCursor::Write { index: 0, };
                    return Instruction::Perform(Perform {
                        op: Op::BlockStart,
                        level_index: cursor.level_index,
                        block_index: cursor.block_index,
                        next: Script { inner: Fsm::Busy(self), },
                    });
                },
                BlockCursor::Write { index, } if index < sketch.block_size() => {
                    cursor.block_cursor = BlockCursor::Write { index: index + 1, };
                    cursor.items_remain -= 1;
                    return Instruction::Perform(Perform {
                        op: Op::BlockItem { index, },
                        level_index: cursor.level_index,
                        block_index: cursor.block_index,
                        next: Script { inner: Fsm::Busy(Busy { level_curr: self.level_base, ..self }) },
                    });
                },
                BlockCursor::Write { .. } => {
                    cursor.block_cursor = BlockCursor::Start;
                    let block_index = cursor.block_index;
                    cursor.block_index += 1;
                    return Instruction::Perform(Perform {
                        op: Op::BlockFinish,
                        level_index: cursor.level_index,
                        block_index,
                        next: Script { inner: Fsm::Busy(Busy { level_curr: self.level_curr + 1, ..self }), },
                    });
                },
            }
        } else {
            Instruction::Done
        }
    }
}
