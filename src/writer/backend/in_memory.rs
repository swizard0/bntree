use std::io;
use bincode;
use serde_derive::{
    Serialize,
    Deserialize,
};

use super::super::{
    two_pass,
    super::sketch,
};

const MAGIC: u64 = 0x680f9a7a8b7a680d;

pub fn build<'s>(sketch: &'s sketch::Tree) -> Instruction<'s> {
    let mut storage = Storage {
        memory: Vec::new(),
    };
    let mut cursor = io::Cursor::new(&mut storage.memory);
    bincode::serialize_into(&mut cursor, &MAGIC).unwrap();
    bincode::serialize_into(&mut cursor, sketch).unwrap();

    Instruction::PassMarkup(InMemory {
        inner_fsm: two_pass::write_blocks(sketch, 0, storage.memory.len()),
        inner: Inner {
            storage,
            available_blocks: Vec::with_capacity(sketch.levels().len()),
        },
    })
}

pub struct InMemory<'s> {
    inner_fsm: two_pass::WriteBlocks<'s, BlockWriter, BlockWriter, usize>,
    inner: Inner,
}

struct Inner {
    storage: Storage,
    available_blocks: Vec<Vec<u8>>,
}

struct Storage {
    memory: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct BlockHeader {
    items_count: u16,
}

impl<'s> InMemory<'s> {
    pub fn next(mut self) -> Instruction<'s> {
        loop {
            match self.inner_fsm.next() {
                two_pass::Instruction::InitialLevelSize(two_pass::InitialLevelSize { next, .. }) => {
                    let two_pass::AllocMarkupBlock { next, .. } =
                        next.level_header_size(0);
                    self.inner_fsm = next.block_ready(BlockWriter::new(
                        self.inner.available_blocks.pop().unwrap_or_else(Vec::new),
                    ));
                },
                two_pass::Instruction::AllocMarkupBlock(two_pass::AllocMarkupBlock { next, .. }) =>
                    self.inner_fsm = next.block_ready(BlockWriter::new(
                        self.inner.available_blocks.pop().unwrap_or_else(Vec::new),
                    )),
                two_pass::Instruction::WriteMarkupItem(two_pass::WriteMarkupItem { block, next, .. }) =>
                    return Instruction::WriteItem(WriteItem {
                        block_writer: block,
                        next: WriteItemNext {
                            inner_fsm_next: WriteItemPass::Markup(next),
                            inner: self.inner,
                        },
                    }),
                two_pass::Instruction::FinishMarkupBlock(two_pass::FinishMarkupBlock { block: block_writer, next, .. }) => {
                    let items_total = block_writer.items_count as u32;
                    let mut block_memory = block_writer.into_inner();
                    let mut rewrite_cursor = io::Cursor::new(&mut block_memory);
                    bincode::serialize_into(&mut rewrite_cursor, &items_total).unwrap();
                    return Instruction::FlushBlock(FlushBlockNext {
                        inner_fsm_next: FlushBlockPass::Markup(next),
                        block: block_memory,
                        inner: self.inner,
                    });
                },
                two_pass::Instruction::WriteTreeHeader(two_pass::WriteTreeHeader { offset, tree_total_size, next, }) => {
                    assert_eq!(offset, 0);
                    let tree_header_size = self.inner.storage.memory.len();
                    assert!(tree_total_size >= tree_header_size);
                    self.inner.storage.memory.resize(tree_total_size, 0xff);
                    return Instruction::PassWrite(InMemory {
                        inner_fsm: next.tree_header_written(tree_header_size),
                        inner: self.inner,
                    });
                },
                two_pass::Instruction::WriteLevelHeader(two_pass::WriteLevelHeader { next, .. }) => {
                    let two_pass::WriteBlockHeader { next, .. } =
                        next.level_header_written(0);
                    let block_writer = BlockWriter::new(
                        self.inner.available_blocks.pop().unwrap_or_else(Vec::new),
                    );
                    let header_size = block_writer.header_size;
                    self.inner_fsm = next.block_header_written(block_writer, header_size);
                },
                two_pass::Instruction::WriteBlockHeader(two_pass::WriteBlockHeader { next, .. }) => {
                    let block_writer = BlockWriter::new(
                        self.inner.available_blocks.pop().unwrap_or_else(Vec::new),
                    );
                    let header_size = block_writer.header_size;
                    self.inner_fsm = next.block_header_written(block_writer, header_size);
                },
                two_pass::Instruction::WriteBlockItem(two_pass::WriteBlockItem { block_meta, child_block_offset, next, .. }) => {
                    let current_block_size = block_meta.get_ref().len();
                    return Instruction::WriteItem(WriteItem {
                        block_writer: block_meta,
                        next: WriteItemNext {
                            inner_fsm_next: WriteItemPass::Write {
                                next,
                                child_block_offset,
                                current_block_size,
                            },
                            inner: self.inner,
                        },
                    });
                },
                two_pass::Instruction::FlushBlock(two_pass::FlushBlock {
                    block_meta: block_writer,
                    block_start_offset,
                    block_end_offset,
                    next,
                    ..
                }) => {
                    let items_total = block_writer.items_count as u32;
                    let mut block_memory = block_writer.into_inner();
                    let mut rewrite_cursor = io::Cursor::new(&mut block_memory);
                    bincode::serialize_into(&mut rewrite_cursor, &items_total).unwrap();
                    return Instruction::FlushBlock(FlushBlockNext {
                        inner_fsm_next: FlushBlockPass::Write {
                            block_start_offset,
                            block_end_offset,
                            next,
                        },
                        block: block_memory,
                        inner: self.inner,
                    });
                },
                two_pass::Instruction::Done =>
                    return Instruction::Done(self.inner.storage.memory),
            }
        }
    }
}

pub enum Instruction<'s> {
    PassMarkup(InMemory<'s>),
    PassWrite(InMemory<'s>),
    WriteItem(WriteItem<'s>),
    FlushBlock(FlushBlockNext<'s>),
    Done(Vec<u8>),
}

pub struct WriteItem<'s> {
    pub block_writer: BlockWriter,
    pub next: WriteItemNext<'s>,
}

pub struct WriteItemNext<'s> {
    inner_fsm_next: WriteItemPass<'s>,
    inner: Inner,
}

enum WriteItemPass<'s> {
    Markup(two_pass::WriteMarkupItemNext<'s, BlockWriter, BlockWriter, usize>),
    Write {
        next: two_pass::WriteBlockItemNext<'s, BlockWriter, BlockWriter, usize>,
        current_block_size: usize,
        child_block_offset: Option<usize>,
    }
}

impl<'s> WriteItemNext<'s> {
    pub fn item_written(self, mut block_writer: BlockWriter) -> InMemory<'s> {
        match self.inner_fsm_next {
            WriteItemPass::Markup(next) => {
                let no_child: Option<usize> = None;
                bincode::serialize_into(&mut block_writer, &no_child).unwrap();
                block_writer.items_count += 1;
                InMemory {
                    inner_fsm: next.item_written(block_writer),
                    inner: self.inner,
                }
            },
            WriteItemPass::Write { next, child_block_offset, current_block_size, } => {
                bincode::serialize_into(&mut block_writer, &child_block_offset).unwrap();
                let written = block_writer.get_ref().len() - current_block_size;
                block_writer.items_count += 1;
                InMemory {
                    inner_fsm: next.item_written(block_writer, written),
                    inner: self.inner,
                }
            },
        }
    }
}


pub struct FlushBlockNext<'s> {
    inner_fsm_next: FlushBlockPass<'s>,
    block: Vec<u8>,
    inner: Inner,
}

enum FlushBlockPass<'s> {
    Markup(two_pass::FinishMarkupBlockNext<'s, BlockWriter, BlockWriter, usize>),
    Write {
        block_start_offset: usize,
        block_end_offset: usize,
        next: two_pass::FlushBlockNext<'s, BlockWriter, BlockWriter, usize>,
    },
}

impl<'s> FlushBlockNext<'s> {
    pub fn block_bytes(&self) -> &[u8] {
        &self.block
    }

    pub fn block_flushed(mut self) -> InMemory<'s> {
        let block_size = self.block.len();
        let mut next = match self.inner_fsm_next {
            FlushBlockPass::Markup(next) =>
                InMemory {
                    inner_fsm: next.block_finished(block_size),
                    inner: self.inner,
                },
            FlushBlockPass::Write { block_start_offset, block_end_offset, next, } => {
                assert_eq!(block_size, block_end_offset - block_start_offset);
                let flush_area = &mut self.inner.storage
                    .memory[block_start_offset .. block_end_offset];
                flush_area.copy_from_slice(&self.block);
                InMemory {
                    inner_fsm: next.block_flushed(),
                    inner: self.inner,
                }
            },
        };
        next.inner.available_blocks.push(self.block);
        next
    }

    pub fn modified_block_flushed(mut self, modified_memory: &[u8]) -> InMemory<'s> {
        self.inner.available_blocks.push(self.block);
        match self.inner_fsm_next {
            FlushBlockPass::Markup(next) =>
                InMemory {
                    inner_fsm: next.block_finished(modified_memory.len()),
                    inner: self.inner,
                },
            FlushBlockPass::Write { block_start_offset, block_end_offset, next, } => {
                assert_eq!(modified_memory.len(), block_end_offset - block_start_offset);
                let flush_area = &mut self.inner.storage
                    .memory[block_start_offset .. block_end_offset];
                flush_area.copy_from_slice(modified_memory);
                InMemory {
                    inner_fsm: next.block_flushed(),
                    inner: self.inner,
                }
            },
        }
    }
}


pub struct BlockWriter {
    cursor: io::Cursor<Vec<u8>>,
    header_size: usize,
    items_count: usize,
}

impl BlockWriter {
    fn new(mut block: Vec<u8>) -> BlockWriter {
        block.clear();
        let mut cursor = io::Cursor::new(block);
        bincode::serialize_into(&mut cursor, &0u32).unwrap();
        let header_size = cursor.get_ref().len();
        BlockWriter { cursor, header_size, items_count: 0, }
    }

    fn get_ref(&self) -> &[u8] {
        self.cursor.get_ref()
    }

    fn into_inner(self) -> Vec<u8> {
        self.cursor.into_inner()
    }
}

impl io::Write for BlockWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.cursor.write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.cursor.write_vectored(bufs)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.cursor.flush()
    }
}
