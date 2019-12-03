use super::super::sketch;

pub fn build() -> Instruction {
    Instruction::TreeStart { next: Plan { inner: Mode::Init, }, }
}

pub enum Instruction {
    TreeStart { next: Plan, },
    BlockStart { level_index: usize, block_index: usize, next: Plan, },
    WriteItem { level_index: usize, block_index: usize, item_index: usize, next: Plan, },
    BlockFinish { level_index: usize, block_index: usize, next: Plan, },
    Done,
}

pub struct Plan {
    inner: Mode,
}

enum Mode {
    Init,
    Progress {
        cursors: Vec<LevelCursor>,
        level_curr: usize,
        level_base: usize,
    },
    Done,
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

impl Plan {
    pub fn next(mut self, sketch: &sketch::Tree) -> Instruction {
        loop {
            match self.inner {
                Mode::Init =>
                    self.inner = Mode::Progress {
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
                Mode::Progress { ref cursors, level_curr, .. } if level_curr >= cursors.len() =>
                    self.inner = Mode::Done,
                Mode::Progress { mut cursors, level_curr, level_base, } => {
                    let cursor = &mut cursors[level_curr];
                    if cursor.items_remain == 0 {
                        return Instruction::BlockFinish {
                            level_index: cursor.level_index,
                            block_index: cursor.block_index,
                            next: Plan {
                                inner: Mode::Progress {
                                    cursors,
                                    level_base: level_base + 1,
                                    level_curr: level_base + 1,
                                },
                            },
                        };
                    }
                    match cursor.block_cursor {
                        BlockCursor::Start => {
                            cursor.block_cursor = BlockCursor::Write { index: 0, };
                            return Instruction::BlockStart {
                                level_index: cursor.level_index,
                                block_index: cursor.block_index,
                                next: Plan { inner: Mode::Progress { cursors, level_curr, level_base, }, },
                            };
                        },
                        BlockCursor::Write { index, } if index < sketch.block_size() => {
                            cursor.block_cursor = BlockCursor::Write { index: index + 1, };
                            cursor.items_remain -= 1;
                            return Instruction::WriteItem {
                                level_index: cursor.level_index,
                                block_index: cursor.block_index,
                                item_index: index,
                                next: Plan { inner: Mode::Progress { cursors, level_curr: level_base, level_base, }, },
                            };
                        },
                        BlockCursor::Write { .. } => {
                            cursor.block_cursor = BlockCursor::Start;
                            let block_index = cursor.block_index;
                            cursor.block_index += 1;
                            return Instruction::BlockFinish {
                                level_index: cursor.level_index,
                                block_index,
                                next: Plan { inner: Mode::Progress { cursors, level_curr: level_curr + 1, level_base, }, },
                            };
                        },
                    }
                },
                Mode::Done =>
                    return Instruction::Done,
            }
        }
    }
}
