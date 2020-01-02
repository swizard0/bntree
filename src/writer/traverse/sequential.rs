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
    FinishTreeHeader(FinishTreeHeader<'ctx>),
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
    InvalidContextForStepMarkup,
    InvalidContextForStepWrite,
    InvalidContextModeForUnderlyingAction,
    InvalidContextMarkupStateForUnderlyingAction,
    InvalidContextWriteStateForUnderlyingAction,
    InvalidContextModeForWriteTreeHeader,
    InvalidContextModeForFinishTreeHeader,
    StepRecMarkup(two_pass::markup::Error),
    StepRecWrite(two_pass::write::Error<Offset>),
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
        match &mut context.mode {
            Mode::None =>
                Ok(Instruction::Op(Op::MarkupStart(MarkupStart {
                    next: self,
                }))),

            Mode::TreeHeaderWrite(Pass::Markup(ModeMarkupHeaderWrite { markup_context, active_block_writer, })) =>
                Ok(Instruction::Op(Op::WriteTreeHeader(WriteTreeHeader {
                    block_writer: active_block_writer,
                    next: WriteTreeHeaderNext {
                        script: self,
                    },
                }))),

            Mode::TreeHeaderWrite(Pass::Write(ModeWriteHeaderWrite)) =>
                unimplemented!(),

            Mode::TreeHeaderFinish(Pass::Markup(ModeMarkupHeaderFinish { markup_context, active_block, })) =>
                Ok(Instruction::Op(Op::FinishTreeHeader(FinishTreeHeader {
                    block: active_block,
                    next: FinishTreeHeaderNext {
                        script: self,
                    },
                }))),


            Mode::TreeHeaderFinish(Pass::Write(ModeWriteHeaderFinish)) =>
                unimplemented!(),

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
    TreeHeaderWrite(Pass<ModeMarkupHeaderWrite, ModeWriteHeaderWrite>),
    TreeHeaderFinish(Pass<ModeMarkupHeaderFinish, ModeWriteHeaderFinish>),
    Step(Pass<ModeMarkup, ModeWrite>),
}


struct ModeMarkupHeaderWrite {
    markup_context: two_pass::markup::Context<BlockWriter, Offset>,
    active_block_writer: BlockWriter,
}

struct ModeWriteHeaderWrite;


struct ModeMarkupHeaderFinish {
    markup_context: two_pass::markup::Context<BlockWriter, Offset>,
    active_block: Vec<u8>,
}

struct ModeWriteHeaderFinish;


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

impl From<InstructionMarkup> for UnderlyingOp {
    fn from(op: InstructionMarkup) -> Self {
        UnderlyingOp(Some(Pass::Markup(op)))
    }
}

impl From<InstructionWrite> for UnderlyingOp {
    fn from(op: InstructionWrite) -> Self {
        UnderlyingOp(Some(Pass::Write(op)))
    }
}

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
        context.mode = Mode::TreeHeaderWrite(Pass::Markup(ModeMarkupHeaderWrite {
            markup_context,
            active_block_writer: BlockWriter::new(
                context.available_blocks.pop().unwrap_or_else(Vec::new),
            ),
        }));
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
                context.mode = Mode::TreeHeaderFinish(Pass::Markup(ModeMarkupHeaderFinish {
                    markup_context,
                    active_block: active_block_writer.into_inner(),
                }));
                Ok(Continue {
                    underlying_action: UnderlyingAction::Idle(UnderlyingOp(None)),
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


pub struct FinishTreeHeader<'ctx> {
    pub block: &'ctx mut Vec<u8>,
    pub next: FinishTreeHeaderNext,
}

pub struct FinishTreeHeaderNext {
    script: Script,
}

impl FinishTreeHeaderNext {
    pub fn tree_header_finished(self, context: &mut Context) -> Result<Continue, Error> {
        match mem::replace(&mut context.mode, Mode::None) {
            Mode::TreeHeaderFinish(Pass::Markup(ModeMarkupHeaderFinish { markup_context, active_block: block_memory, })) => {
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
                Err(Error::InvalidContextModeForFinishTreeHeader),
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


impl Continue {
    pub fn step_rec<'ctx>(self, sequential_context: &'ctx mut Context) -> Result<Instruction<'ctx>, Error> {
        let underlying_op = match self.underlying_action {
            UnderlyingAction::Idle(underlying_op) =>
                underlying_op,
            UnderlyingAction::Step(underlying_step) =>
                match underlying_step.action(sequential_context)? {
                    Pass::Markup(ActionMarkup { context: markup_ctx, next: markup_continue, }) => {
                        markup_continue
                            .step_rec(markup_ctx)
                            .map_err(Error::StepRecMarkup)?
                            .into()
                    },
                    Pass::Write(ActionWrite { context: write_ctx, next: write_continue, }) => {
                        write_continue
                            .step_rec(write_ctx)
                            .map_err(Error::StepRecWrite)?
                            .into()
                    },
                },
        };
        self.next.step(sequential_context, underlying_op)
    }
}
