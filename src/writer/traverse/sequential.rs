use std::{io, mem};

use super::super::{
    two_pass,
};

pub enum Instruction<'ctx> {
    Op(Op<'ctx>),
    Done,
}

pub enum Op<'ctx> {
    MarkupStart(MarkupStart),
    WriteTreeHeader(WriteTreeHeader<'ctx>),
}

pub struct Continue {
    pub underlying_action: UnderlyingAction,
    pub next: Script,
}

pub enum UnderlyingAction {
    Idle(UnderlyingOp),
    Step(UnderlyingActionStep),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    InvalidContextRequestIncomplete,
    InvalidContextForStepMarkup,
    InvalidContextForStepWrite,
    InvalidContextModeForUnderlyingAction,
    InvalidContextMarkupStateForUnderlyingAction,
    InvalidContextWriteStateForUnderlyingAction,
    InvalidContextModeForWriteTreeHeader,
}

pub enum Pass<M, W> {
    Markup(M),
    Write(W),
}

pub type Offset = usize;
pub type ContinueMarkup = two_pass::markup::Continue<BlockWriter, Offset>;
pub type ContinueWrite = two_pass::write::Continue<BlockWriter, Offset>;

pub struct Context {
    mode: Mode,
    available_blocks: Vec<Vec<u8>>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            mode: Mode::None,
            available_blocks: Vec::new(),
        }
    }
}

pub struct Script(());

impl Script {
    pub fn boot() -> Continue {
        Continue {
            underlying_action: UnderlyingAction::Idle(
                UnderlyingOp(None),
            ),
            next: Script(()),
        }
    }

    pub fn step<'ctx>(self, context: &'ctx mut Context, op: UnderlyingOp) -> Result<Instruction<'ctx>, Error> {
        match mem::replace(&mut context.mode, Mode::None) {
            Mode::None =>
                Ok(Instruction::Op(Op::MarkupStart(MarkupStart {
                    next: self,
                }))),

            Mode::TreeHeaderWait(Pass::Markup(ModeMarkupHeaderWait { markup_context, })) => {
                context.mode = Mode::TreeHeaderWrite(Pass::Markup(ModeMarkupHeaderWrite {
                    markup_context,
                    active_block_writer: BlockWriter::new(
                        context.available_blocks.pop().unwrap_or_else(Vec::new),
                    ),
                }));
                Ok(Instruction::Op(Op::WriteTreeHeader(WriteTreeHeader {
                    block_writer: if let Mode::TreeHeaderWrite(
                        Pass::Markup(ModeMarkupHeaderWrite { ref mut active_block_writer, .. }),
                    ) = context.mode {
                        active_block_writer
                    } else {
                        unreachable!()
                    },
                    next: WriteTreeHeaderNext {
                        script: self,
                    },
                })))
            },

            Mode::TreeHeaderWait(Pass::Write(ModeWriteHeaderWait)) =>
                unimplemented!(),

            Mode::TreeHeaderWrite(..) =>
                Err(Error::InvalidContextRequestIncomplete),

            Mode::Step(Pass::Markup(ModeMarkup { context, tree_header_size, })) =>
                match op {
                    UnderlyingOp(None) | UnderlyingOp(Some(Pass::Write(..))) =>
                        Err(Error::InvalidContextModeForUnderlyingAction),
                    UnderlyingOp(Some(Pass::Markup(
                        two_pass::markup::Instruction::Op(two_pass::markup::Op::LevelHeaderSize(
                            two_pass::markup::LevelHeaderSize { next, .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Markup(
                        two_pass::markup::Instruction::Op(two_pass::markup::Op::AllocBlock(
                            two_pass::markup::AllocBlock { next, .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Markup(
                        two_pass::markup::Instruction::Op(two_pass::markup::Op::WriteItem(
                            two_pass::markup::WriteItem { next, .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Markup(
                        two_pass::markup::Instruction::Op(two_pass::markup::Op::FinishBlock(
                            two_pass::markup::FinishBlock { next, .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Markup(two_pass::markup::Instruction::Done))) =>
                        unimplemented!(),
                },

            Mode::Step(Pass::Write(ModeWrite { context, })) =>
                match op {
                    UnderlyingOp(None) | UnderlyingOp(Some(Pass::Markup(..))) =>
                        Err(Error::InvalidContextModeForUnderlyingAction),
                    UnderlyingOp(Some(Pass::Write(
                        two_pass::write::Instruction::Op(two_pass::write::Op::WriteTreeHeader(
                            two_pass::write::WriteTreeHeader { next, .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Write(
                        two_pass::write::Instruction::Op(two_pass::write::Op::WriteLevelHeader(
                            two_pass::write::WriteLevelHeader { next, .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Write(
                        two_pass::write::Instruction::Op(two_pass::write::Op::WriteBlockHeader(
                            two_pass::write::WriteBlockHeader { next, .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Write(
                        two_pass::write::Instruction::Op(two_pass::write::Op::WriteItem(
                            two_pass::write::WriteItem { next, .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Write(
                        two_pass::write::Instruction::Op(two_pass::write::Op::FlushBlock(
                            two_pass::write::FlushBlock { next, .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Write(two_pass::write::Instruction::Done))) =>
                        unimplemented!(),
                },
        }
    }
}

enum Mode {
    None,
    TreeHeaderWait(Pass<ModeMarkupHeaderWait, ModeWriteHeaderWait>),
    TreeHeaderWrite(Pass<ModeMarkupHeaderWrite, ModeWriteHeaderWrite>),
    Step(Pass<ModeMarkup, ModeWrite>),
}


struct ModeMarkupHeaderWait {
    markup_context: two_pass::markup::Context<BlockWriter, Offset>,
}

struct ModeWriteHeaderWait;


struct ModeMarkupHeaderWrite {
    markup_context: two_pass::markup::Context<BlockWriter, Offset>,
    active_block_writer: BlockWriter,
}

struct ModeWriteHeaderWrite;


struct ModeMarkup {
    context: two_pass::markup::Context<BlockWriter, Offset>,
    tree_header_size: usize,
}

struct ModeWrite {
    context: two_pass::write::Context<BlockWriter, Offset>,
}

type InstructionMarkup = two_pass::markup::Instruction<BlockWriter, Offset>;
type InstructionWrite = two_pass::write::Instruction<BlockWriter, Offset>;

pub struct UnderlyingOp(Option<Pass<InstructionMarkup, InstructionWrite>>);

pub struct UnderlyingActionStep(Pass<ContinueMarkup, ContinueWrite>);

impl UnderlyingActionStep {
    pub fn action<'a>(self, context: &'a mut Context) -> Result<Pass<ActionMarkup<'a>, ActionWrite<'a>>, Error> {
        match self.0 {
            Pass::Markup(continue_markup) =>
                match &mut context.mode {
                    Mode::Step(Pass::Markup(ModeMarkup { context, .. })) =>
                        Ok(Pass::Markup(ActionMarkup {
                            context,
                            next: continue_markup,
                        })),
                    _ =>
                        Err(Error::InvalidContextModeForUnderlyingAction),
                },
            Pass::Write(continue_write) =>
                match &mut context.mode {
                    Mode::Step(Pass::Write(ModeWrite { context, .. })) =>
                        Ok(Pass::Write(ActionWrite {
                            context,
                            next: continue_write,
                        })),
                    _ =>
                        Err(Error::InvalidContextModeForUnderlyingAction),
                },
        }
    }
}

pub struct ActionMarkup<'a> {
    pub context: &'a mut two_pass::markup::Context<BlockWriter, Offset>,
    pub next: ContinueMarkup,
}

pub struct ActionWrite<'a> {
    pub context: &'a mut two_pass::write::Context<BlockWriter, Offset>,
    pub next: ContinueWrite,
}


pub struct MarkupStart {
    next: Script,
}

impl MarkupStart {
    pub fn created_context(self, markup_context: two_pass::markup::Context<BlockWriter, Offset>, context: &mut Context) -> Continue {
        context.mode = Mode::TreeHeaderWait(Pass::Markup(ModeMarkupHeaderWait { markup_context, }));
        Continue {
            underlying_action: UnderlyingAction::Idle(UnderlyingOp(None)),
            next: self.next,
        }
    }
}


pub struct WriteTreeHeader<'ctx> {
    pub block_writer: &'ctx mut BlockWriter,
    pub next: WriteTreeHeaderNext,
}

pub struct WriteTreeHeaderNext {
    script: Script,
}

impl WriteTreeHeaderNext {
    pub fn tree_header_written(self, context: &mut Context) -> Result<Continue, Error> {
        match mem::replace(&mut context.mode, Mode::None) {
            Mode::TreeHeaderWrite(Pass::Markup(ModeMarkupHeaderWrite { markup_context, active_block_writer, })) => {
                let block_memory = active_block_writer.into_inner();
                let tree_header_size = block_memory.len();
                context.mode = Mode::Step(Pass::Markup(ModeMarkup {
                    context: markup_context,
                    tree_header_size,
                }));
                context.available_blocks.push(block_memory);
                Ok(Continue {
                    underlying_action: UnderlyingAction::Step(
                        UnderlyingActionStep(Pass::Markup(two_pass::markup::Script::boot())),
                    ),
                    next: self.script,
                })
            },
            Mode::TreeHeaderWrite(Pass::Write(..)) =>
                unimplemented!(),
            _ =>
                Err(Error::InvalidContextModeForWriteTreeHeader),
        }
    }
}


pub struct BlockWriter {
    cursor: io::Cursor<Vec<u8>>,
}

impl BlockWriter {
    fn new(mut block: Vec<u8>) -> BlockWriter {
        block.clear();
        BlockWriter { cursor: io::Cursor::new(block), }
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
