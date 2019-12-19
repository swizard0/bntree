use std::io;

use super::super::{
    two_pass,
};

pub enum Instruction {
    Op(Op),
    Done,
}

pub enum Op {
    MarkupStart(MarkupStart),
}

pub struct Continue {
    pub underlying_action: UnderlyingAction,
    pub next: Script,
}

pub enum UnderlyingAction {
    Idle(UnderlyingOp),
    Step(UnderlyingActionStep),
}

pub struct UnderlyingOp(());

#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    InvalidContextForStepMarkup,
    InvalidContextForStepWrite,
    InvalidContextModeForUnderlyingAction,
    InvalidContextMarkupStateForUnderlyingAction,
    InvalidContextWriteStateForUnderlyingAction,
}

pub type Offset = usize;

pub struct VecBlock {
    memory: Vec<u8>,
}

pub struct Context {
    mode: Option<Mode>,
    available_blocks: Vec<Vec<u8>>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            mode: None,
            available_blocks: Vec::new(),
        }
    }
}

pub struct Script(());

impl Script {
    pub fn boot() -> Continue {
        Continue {
            underlying_action: UnderlyingAction::Idle(
                UnderlyingOp(()),
            ),
            next: Script(()),
        }
    }

    pub fn step(self, context: &mut Context, op: UnderlyingOp) -> Result<Instruction, Error> {
        match context.mode.take() {
            None =>
                Ok(Instruction::Op(Op::MarkupStart(MarkupStart {
                    next: self,
                }))),
            Some(Mode::Markup {
                state: UnderlyingMarkupState::Op(
                    two_pass::markup::Instruction::Op(two_pass::markup::Op::InitialLevelSize(
                        two_pass::markup::InitialLevelSize { next, .. },
                    )),
                ),
                ..
            }) =>
                unimplemented!(),
            Some(Mode::Markup {
                state: UnderlyingMarkupState::Op(
                    two_pass::markup::Instruction::Op(two_pass::markup::Op::AllocBlock(
                        two_pass::markup::AllocBlock { next, .. },
                    )),
                ),
                ..
            }) =>
                unimplemented!(),
            Some(Mode::Markup {
                state: UnderlyingMarkupState::Op(
                    two_pass::markup::Instruction::Op(two_pass::markup::Op::WriteItem(
                        two_pass::markup::WriteItem { next, .. },
                    )),
                ),
                ..
            }) =>
                unimplemented!(),
            Some(Mode::Markup {
                state: UnderlyingMarkupState::Op(
                    two_pass::markup::Instruction::Op(two_pass::markup::Op::FinishBlock(
                        two_pass::markup::FinishBlock { next, .. },
                    )),
                ),
                ..
            }) =>
                unimplemented!(),
            Some(Mode::Markup {
                state: UnderlyingMarkupState::Op(two_pass::markup::Instruction::Done),
                ..
            }) =>
                unimplemented!(),
            Some(Mode::Markup { state: UnderlyingMarkupState::Step(..), .. }) =>
                Err(Error::InvalidContextForStepMarkup),
            Some(Mode::Write {
                state: UnderlyingWriteState::Op(
                    two_pass::write::Instruction::Op(two_pass::write::Op::WriteTreeHeader(
                        two_pass::write::WriteTreeHeader { next, .. },
                    )),
                ),
                ..
            }) =>
                unimplemented!(),
            Some(Mode::Write {
                state: UnderlyingWriteState::Op(
                    two_pass::write::Instruction::Op(two_pass::write::Op::WriteLevelHeader(
                        two_pass::write::WriteLevelHeader { next, .. },
                    )),
                ),
                ..
            }) =>
                unimplemented!(),
            Some(Mode::Write {
                state: UnderlyingWriteState::Op(
                    two_pass::write::Instruction::Op(two_pass::write::Op::WriteBlockHeader(
                        two_pass::write::WriteBlockHeader { next, .. },
                    )),
                ),
                ..
            }) =>
                unimplemented!(),
            Some(Mode::Write {
                state: UnderlyingWriteState::Op(
                    two_pass::write::Instruction::Op(two_pass::write::Op::WriteItem(
                        two_pass::write::WriteItem { next, .. },
                    )),
                ),
                ..
            }) =>
                unimplemented!(),
            Some(Mode::Write {
                state: UnderlyingWriteState::Op(
                    two_pass::write::Instruction::Op(two_pass::write::Op::FlushBlock(
                        two_pass::write::FlushBlock { next, .. },
                    )),
                ),
                ..
            }) =>
                unimplemented!(),
            Some(Mode::Write {
                state: UnderlyingWriteState::Op(two_pass::write::Instruction::Done),
                ..
            }) =>
                unimplemented!(),
            Some(Mode::Write { state: UnderlyingWriteState::Step(..), .. }) =>
                Err(Error::InvalidContextForStepWrite),
        }
    }
}


enum Mode {
    Markup {
        context: two_pass::markup::Context<VecBlock, Offset>,
        state: UnderlyingMarkupState,
    },
    Write {
        context: two_pass::write::Context<VecBlock, Offset>,
        state: UnderlyingWriteState,
    },
}

enum UnderlyingMarkupState {
    Op(two_pass::markup::Instruction<VecBlock, Offset>),
    Step(two_pass::markup::Continue<VecBlock, Offset>),
}

enum UnderlyingWriteState {
    Op(two_pass::write::Instruction<VecBlock, Offset>),
    Step(two_pass::write::Continue<VecBlock, Offset>),
}


pub struct UnderlyingActionStep(());

impl UnderlyingActionStep {
    pub fn action<'a>(self, context: &'a mut Context) -> Result<UnderlyingActionStepKind<'a>, Error> {
        use std::mem;
        match &mut context.mode {
            None =>
                Err(Error::InvalidContextModeForUnderlyingAction),
            Some(Mode::Markup { context, state: UnderlyingMarkupState::Step(..), }) =>
                Err(Error::InvalidContextMarkupStateForUnderlyingAction),
            Some(Mode::Markup { context, state: UnderlyingMarkupState::Op(op), }) =>
                Ok(UnderlyingActionStepKind::Markup {
                    context,
                    op: mem::replace(op, two_pass::markup::Instruction::Done),
                }),
            Some(Mode::Write { context, state: UnderlyingWriteState::Step(..), }) =>
                Err(Error::InvalidContextWriteStateForUnderlyingAction),
            Some(Mode::Write { context, state: UnderlyingWriteState::Op(op), }) =>
                Ok(UnderlyingActionStepKind::Write {
                    context,
                    op: mem::replace(op, two_pass::write::Instruction::Done),
                }),
        }
    }
}

pub enum UnderlyingActionStepKind<'a> {
    Markup {
        context: &'a mut two_pass::markup::Context<VecBlock, Offset>,
        op: two_pass::markup::Instruction<VecBlock, Offset>,
    },
    Write {
        context: &'a mut two_pass::write::Context<VecBlock, Offset>,
        op: two_pass::write::Instruction<VecBlock, Offset>,
    },
}


pub struct MarkupStart {
    next: Script,
}

impl MarkupStart {
    pub fn created_context(self, context: &mut Context, markup_context: two_pass::markup::Context<VecBlock, Offset>) -> Continue {
        context.mode = Some(Mode::Markup {
            context: markup_context,
            state: UnderlyingMarkupState::Step(
                two_pass::markup::Script::boot(),
            ),
        });
        Continue {
            underlying_action: UnderlyingAction::Step(UnderlyingActionStep(())),
            next: self.next,
        }
    }
}


pub struct BlockWriter {
    cursor: io::Cursor<Vec<u8>>,
}

impl BlockWriter {
    fn new(mut block: Vec<u8>) -> BlockWriter {
        block.clear();
        let mut cursor = io::Cursor::new(block);
        BlockWriter { cursor, }
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
