use super::super::sketch;

pub fn build<'s>(sketch: &'s sketch::Tree) -> Plan<'s> {
    Plan {
        sketch,
        cursors: sketch
            .levels()
            .iter()
            .rev()
            .map(|level| LevelCursor {
                level,
                block_index: 0,
                block_cursor: BlockCursor::Start,
                items_remain: level.items_count,
            })
            .collect(),
        level_curr: 0,
        level_base: 0,
    }
}

pub struct Plan<'s> {
    pub sketch: &'s sketch::Tree,
    cursors: Vec<LevelCursor<'s>>,
    level_curr: usize,
    level_base: usize,
}

struct LevelCursor<'s> {
    level: &'s sketch::Level,
    block_index: usize,
    block_cursor: BlockCursor,
    items_remain: usize,
}

enum BlockCursor {
    Start,
    Write { index: usize, },
}

impl<'s> Iterator for Plan<'s> {
    type Item = Instruction<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.level_curr < self.cursors.len() {
            let cursor = &mut self.cursors[self.level_curr];
            if cursor.items_remain == 0 {
                let instruction = Instruction {
                    level: cursor.level,
                    block_index: cursor.block_index,
                    op: Op::BlockFinish,
                };
                self.level_base += 1;
                self.level_curr = self.level_base;
                return Some(instruction);
            }
            match cursor.block_cursor {
                BlockCursor::Start => {
                    cursor.block_cursor = BlockCursor::Write { index: 0, };
                    return Some(Instruction {
                        level: cursor.level,
                        block_index: cursor.block_index,
                        op: Op::BlockStart,
                    });
                },
                BlockCursor::Write { index, } if index < self.sketch.block_size() => {
                    cursor.block_cursor = BlockCursor::Write { index: index + 1, };
                    cursor.items_remain -= 1;
                    self.level_curr = self.level_base;
                    return Some(Instruction {
                        level: cursor.level,
                        block_index: cursor.block_index,
                        op: Op::WriteItem {
                            block_item_index: index,
                        },
                    });
                },
                BlockCursor::Write { .. } => {
                    cursor.block_cursor = BlockCursor::Start;
                    let block_index = cursor.block_index;
                    cursor.block_index += 1;
                    self.level_curr += 1;
                    return Some(Instruction {
                        level: cursor.level,
                        block_index,
                        op: Op::BlockFinish,
                    });
                },
            }
        }

        None
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Instruction<'s> {
    pub level: &'s sketch::Level,
    pub block_index: usize,
    pub op: Op,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Op {
    BlockStart,
    WriteItem { block_item_index: usize, },
    BlockFinish,
}
