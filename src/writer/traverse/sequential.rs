use std::io;

use super::super::two_pass;

pub enum Instruction<'ctx> {
    Op(Op<'ctx>),
    Noop(Continue),
    Done,
}

pub enum Op<'ctx> {
    MarkupStart(MarkupStart),
    WriteTreeHeader(WriteTreeHeader<'ctx>),
    FinishTreeHeader(FinishTreeHeader<'ctx>),
    WriteBlockHeader(WriteBlockHeader<'ctx>),
    FinishBlockHeader(FinishBlockHeader<'ctx>),
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
    ContinuationFinishRequired,
    ContextModesOutOfSync,
    ScriptStepRequired,
    InvalidContextForStepMarkup,
    InvalidContextForStepWrite,
    InvalidContextModeForUnderlyingAction,
    InvalidContextMarkupStateForUnderlyingAction,
    InvalidContextWriteStateForUnderlyingAction,
    InvalidContextModeForMarkupStart,
    InvalidContextModeForWriteTreeHeader,
    InvalidContextModeForFinishTreeHeader,
    InvalidContextModeForWriteBlockHeader,
    InvalidContextModeForFinishBlockHeader,
    Markup(two_pass::markup::Error),
    ContinueMarkup(two_pass::markup::Error),
    ContinueWrite(two_pass::write::Error<Offset>),
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
    mode_script: Option<ModeScript>,
    mode_continue: Option<ModeContinue>,
    available_blocks: Vec<Vec<u8>>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            mode_script: Some(ModeScript::Boot),
            mode_continue: None,
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
        if context.mode_continue.is_some() {
            return Err(Error::ContinuationFinishRequired);
        }

        context.mode_continue = Some(match context.mode_script.take() {
            None =>
                return Err(Error::ContinuationFinishRequired),

            Some(ModeScript::Boot) =>
                ModeContinue::MarkupStart,

            Some(ModeScript::TreeHeaderWrite(Pass::Markup(markup_context))) =>
                ModeContinue::TreeHeaderWrite(
                    Pass::Markup(ModeMarkupWrite {
                        total_size: 0,
                        markup_context,
                        active_block_writer: BlockWriter::new_blank(
                            context.available_blocks.pop().unwrap_or_else(Vec::new),
                        ),
                    })),

            Some(ModeScript::TreeHeaderWrite(Pass::Write(..))) =>
                unimplemented!(),

            Some(ModeScript::TreeHeaderFinish(pass)) =>
                ModeContinue::TreeHeaderFinish(pass),

            Some(ModeScript::BlockHeaderFinish(pass)) =>
                ModeContinue::BlockHeaderFinish(pass),

            Some(ModeScript::Step(Pass::Markup(ModeMarkup { context: mut markup_context, total_size, }))) =>
                match op {
                    UnderlyingOp(None) | UnderlyingOp(Some(Pass::Write(..))) =>
                        return Err(Error::InvalidContextModeForUnderlyingAction),
                    UnderlyingOp(Some(Pass::Markup(
                        two_pass::markup::Instruction::Op(two_pass::markup::Op::LevelHeaderSize(
                            two_pass::markup::LevelHeaderSize { next, .. },
                        )),
                    ))) => {
                        let markup_next = next.level_header_size(0, &mut markup_context)
                            .map_err(Error::Markup)?;
                        context.mode_script = Some(ModeScript::Step(
                            Pass::Markup(ModeMarkup { context: markup_context, total_size, }),
                        ));
                        return Ok(Instruction::Noop(Continue {
                            underlying_action: UnderlyingAction::Step(
                                UnderlyingActionStep(Pass::Markup(markup_next)),
                            ),
                            next: self,
                        }))
                    },
                    UnderlyingOp(Some(Pass::Markup(
                        two_pass::markup::Instruction::Op(two_pass::markup::Op::AllocBlock(
                            two_pass::markup::AllocBlock { items_count, next, .. },
                        )),
                    ))) =>
                        ModeContinue::BlockHeaderWrite(Pass::Markup(ModeMarkupWriteBlockHeader {
                            base: ModeMarkupWrite {
                                total_size,
                                markup_context,
                                active_block_writer: BlockWriter::new_blank(
                                    context.available_blocks.pop().unwrap_or_else(Vec::new),
                                ),
                            },
                            items_count,
                            next,
                        })),
                    UnderlyingOp(Some(Pass::Markup(
                        two_pass::markup::Instruction::Op(two_pass::markup::Op::WriteItem(
                            two_pass::markup::WriteItem { .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Markup(
                        two_pass::markup::Instruction::Op(two_pass::markup::Op::FinishBlock(
                            two_pass::markup::FinishBlock { .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Markup(two_pass::markup::Instruction::Done))) =>
                        unimplemented!(),
                },

            Some(ModeScript::Step(Pass::Write(ModeWrite { .. }))) =>
                match op {
                    UnderlyingOp(None) | UnderlyingOp(Some(Pass::Markup(..))) =>
                        return Err(Error::InvalidContextModeForUnderlyingAction),
                    UnderlyingOp(Some(Pass::Write(
                        two_pass::write::Instruction::Op(two_pass::write::Op::WriteTreeHeader(
                            two_pass::write::WriteTreeHeader { .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Write(
                        two_pass::write::Instruction::Op(two_pass::write::Op::WriteLevelHeader(
                            two_pass::write::WriteLevelHeader { .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Write(
                        two_pass::write::Instruction::Op(two_pass::write::Op::WriteBlockHeader(
                            two_pass::write::WriteBlockHeader { .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Write(
                        two_pass::write::Instruction::Op(two_pass::write::Op::WriteItem(
                            two_pass::write::WriteItem { .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Write(
                        two_pass::write::Instruction::Op(two_pass::write::Op::FlushBlock(
                            two_pass::write::FlushBlock { .. },
                        )),
                    ))) =>
                        unimplemented!(),
                    UnderlyingOp(Some(Pass::Write(two_pass::write::Instruction::Done))) =>
                        unimplemented!(),
                },
        });

        match &mut context.mode_continue {
            None =>
                Err(Error::ContextModesOutOfSync),

            Some(ModeContinue::MarkupStart) =>
                Ok(Instruction::Op(Op::MarkupStart(MarkupStart {
                    next: self,
                }))),

            Some(ModeContinue::TreeHeaderWrite(Pass::Markup(ModeMarkupWrite { active_block_writer, .. }))) =>
                Ok(Instruction::Op(Op::WriteTreeHeader(WriteTreeHeader {
                    block_writer: active_block_writer,
                    next: WriteTreeHeaderNext {
                        script: self,
                    },
                }))),

            Some(ModeContinue::TreeHeaderWrite(Pass::Write(ModeWriteWrite))) =>
                unimplemented!(),

            Some(ModeContinue::TreeHeaderFinish(Pass::Markup(ModeMarkupFinish { active_block, .. }))) =>
                Ok(Instruction::Op(Op::FinishTreeHeader(FinishTreeHeader {
                    block: active_block,
                    next: FinishTreeHeaderNext {
                        script: self,
                    },
                }))),

            Some(ModeContinue::TreeHeaderFinish(Pass::Write(ModeWriteFinish))) =>
                unimplemented!(),

            Some(ModeContinue::BlockHeaderWrite(Pass::Markup(ModeMarkupWriteBlockHeader {
                base: ModeMarkupWrite { active_block_writer, .. },
                items_count,
                ..
            }))) =>
                Ok(Instruction::Op(Op::WriteBlockHeader(WriteBlockHeader {
                    items_count: *items_count,
                    block_writer: active_block_writer,
                    next: WriteBlockHeaderNext {
                        script: self,
                    },
                }))),

            Some(ModeContinue::BlockHeaderWrite(Pass::Write(..))) =>
                unimplemented!(),

            Some(ModeContinue::BlockHeaderFinish(Pass::Markup(ModeMarkupFinishBlockHeader { base: ModeMarkupFinish { active_block, .. }, .. }))) =>
                Ok(Instruction::Op(Op::FinishBlockHeader(FinishBlockHeader {
                    block: active_block,
                    next: FinishBlockHeaderNext {
                        script: self,
                    },
                }))),

            Some(ModeContinue::BlockHeaderFinish(Pass::Write(..))) =>
                unimplemented!(),

        }
    }
}

type ContextMarkup = two_pass::markup::Context<BlockWriter, Offset>;
type ContextWrite = two_pass::write::Context<BlockWriter, Offset>;

enum ModeScript {
    Boot,
    TreeHeaderWrite(Pass<ContextMarkup, ContextWrite>),
    TreeHeaderFinish(Pass<ModeMarkupFinish, ModeWriteFinish>),
    Step(Pass<ModeMarkup, ModeWrite>),
    BlockHeaderFinish(Pass<ModeMarkupFinishBlockHeader, ModeWriteFinishBlockHeader>),
}

enum ModeContinue {
    MarkupStart,
    TreeHeaderWrite(Pass<ModeMarkupWrite, ModeWriteWrite>),
    TreeHeaderFinish(Pass<ModeMarkupFinish, ModeWriteFinish>),
    BlockHeaderWrite(Pass<ModeMarkupWriteBlockHeader, ModeWriteWriteBlockHeader>),
    BlockHeaderFinish(Pass<ModeMarkupFinishBlockHeader, ModeWriteFinishBlockHeader>),
}

struct ModeMarkupWrite {
    total_size: usize,
    markup_context: ContextMarkup,
    active_block_writer: BlockWriter,
}

struct ModeWriteWrite;

struct ModeMarkupWriteBlockHeader {
    base: ModeMarkupWrite,
    items_count: usize,
    next: two_pass::markup::AllocBlockNext<BlockWriter, Offset>,
}

struct ModeWriteWriteBlockHeader {
    base: ModeWriteWrite,
}

struct ModeMarkupFinish {
    total_size: usize,
    markup_context: ContextMarkup,
    active_block: Vec<u8>,
}

struct ModeWriteFinish;

struct ModeMarkupFinishBlockHeader {
    base: ModeMarkupFinish,
    next: two_pass::markup::AllocBlockNext<BlockWriter, Offset>,
}

struct ModeWriteFinishBlockHeader {
    base: ModeWriteFinish,
}


struct ModeMarkup {
    context: ContextMarkup,
    total_size: usize,
}

struct ModeWrite {
    context: ContextWrite,
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
                match &mut context.mode_script {
                    Some(ModeScript::Step(Pass::Markup(ModeMarkup { context, .. }))) =>
                        Ok(Pass::Markup(ActionMarkup {
                            context,
                            next: continue_markup,
                        })),
                    _ =>
                        Err(Error::InvalidContextModeForUnderlyingAction),
                },
            Pass::Write(continue_write) =>
                match &mut context.mode_script {
                    Some(ModeScript::Step(Pass::Write(ModeWrite { context, .. }))) =>
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
    pub context: &'a mut ContextMarkup,
    pub next: ContinueMarkup,
}

pub struct ActionWrite<'a> {
    pub context: &'a mut ContextWrite,
    pub next: ContinueWrite,
}


pub struct MarkupStart {
    next: Script,
}

impl MarkupStart {
    pub fn created_context(self, markup_context: ContextMarkup, context: &mut Context) -> Result<Continue, Error> {
        if context.mode_script.is_some() {
            return Err(Error::ScriptStepRequired);
        }
        if let Some(ModeContinue::MarkupStart) = context.mode_continue.take() {
            context.mode_script = Some(ModeScript::TreeHeaderWrite(Pass::Markup(markup_context)));
            Ok(Continue {
                underlying_action: UnderlyingAction::Idle(UnderlyingOp(None)),
                next: self.next,
            })
        } else {
            Err(Error::InvalidContextModeForMarkupStart)
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
        if context.mode_script.is_some() {
            return Err(Error::ScriptStepRequired);
        }
        if let Some(ModeContinue::TreeHeaderWrite(tree_header_write)) = context.mode_continue.take() {
            match tree_header_write {
                Pass::Markup(ModeMarkupWrite { total_size, markup_context, active_block_writer, }) => {
                    context.mode_script = Some(ModeScript::TreeHeaderFinish(Pass::Markup(ModeMarkupFinish {
                        total_size,
                        markup_context,
                        active_block: active_block_writer.into_inner(),
                    })));
                    Ok(Continue {
                        underlying_action: UnderlyingAction::Idle(UnderlyingOp(None)),
                        next: self.script,
                    })
                },
                Pass::Write(..) =>
                    unimplemented!(),
            }
        } else {
            Err(Error::InvalidContextModeForWriteTreeHeader)
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
        if context.mode_script.is_some() {
            return Err(Error::ScriptStepRequired);
        }
        if let Some(ModeContinue::TreeHeaderFinish(tree_header_finish)) = context.mode_continue.take() {
            match tree_header_finish {
                Pass::Markup(ModeMarkupFinish { total_size, markup_context, active_block, }) => {
                    let tree_header_size = active_block.len();
                    context.mode_script = Some(ModeScript::Step(Pass::Markup(ModeMarkup {
                        context: markup_context,
                        total_size: total_size + tree_header_size,
                    })));
                    context.available_blocks.push(active_block);
                    Ok(Continue {
                        underlying_action: UnderlyingAction::Step(
                            UnderlyingActionStep(Pass::Markup(two_pass::markup::Script::boot())),
                        ),
                        next: self.script,
                    })
                },
                Pass::Write(..) =>
                    unimplemented!(),
            }
        } else {
            Err(Error::InvalidContextModeForFinishTreeHeader)
        }
    }
}


pub struct WriteBlockHeader<'ctx> {
    pub items_count: usize,
    pub block_writer: &'ctx mut BlockWriter,
    pub next: WriteBlockHeaderNext,
}

pub struct WriteBlockHeaderNext {
    script: Script,
}

impl WriteBlockHeaderNext {
    pub fn block_header_written(self, context: &mut Context) -> Result<Continue, Error> {
        if context.mode_script.is_some() {
            return Err(Error::ScriptStepRequired);
        }
        if let Some(ModeContinue::BlockHeaderWrite(block_header_write)) = context.mode_continue.take() {
            match block_header_write {
                Pass::Markup(ModeMarkupWriteBlockHeader {
                    base: ModeMarkupWrite { total_size, markup_context, active_block_writer, },
                    next,
                    ..
                }) => {
                    context.mode_script = Some(ModeScript::BlockHeaderFinish(Pass::Markup(ModeMarkupFinishBlockHeader {
                        base: ModeMarkupFinish {
                            total_size,
                            markup_context,
                            active_block: active_block_writer.into_inner(),
                        },
                        next,
                    })));
                    Ok(Continue {
                        underlying_action: UnderlyingAction::Idle(UnderlyingOp(None)),
                        next: self.script,
                    })
                },
                Pass::Write(..) =>
                    unimplemented!(),
            }
        } else {
            Err(Error::InvalidContextModeForWriteBlockHeader)
        }
    }
}


pub struct FinishBlockHeader<'ctx> {
    pub block: &'ctx mut Vec<u8>,
    pub next: FinishBlockHeaderNext,
}

pub struct FinishBlockHeaderNext {
    script: Script,
}

impl FinishBlockHeaderNext {
    pub fn block_header_finished(self, context: &mut Context) -> Result<Continue, Error> {
        if context.mode_script.is_some() {
            return Err(Error::ScriptStepRequired);
        }
        if let Some(ModeContinue::BlockHeaderFinish(block_header_finish)) = context.mode_continue.take() {
            match block_header_finish {
                Pass::Markup(ModeMarkupFinishBlockHeader {
                    base: ModeMarkupFinish { total_size, mut markup_context, active_block, },
                    next,
                }) => {
                    let block_header_size = active_block.len();
                    let markup_next = next.block_ready(BlockWriter::new(active_block), &mut markup_context)
                        .map_err(Error::ContinueMarkup)?;
                    context.mode_script = Some(ModeScript::Step(Pass::Markup(ModeMarkup {
                        context: markup_context,
                        total_size: total_size + block_header_size,
                    })));
                    Ok(Continue {
                        underlying_action: UnderlyingAction::Step(
                            UnderlyingActionStep(Pass::Markup(markup_next)),
                        ),
                        next: self.script,
                    })
                },
                Pass::Write(..) =>
                    unimplemented!(),
            }
        } else {
            Err(Error::InvalidContextModeForFinishBlockHeader)
        }
    }
}



pub struct BlockWriter {
    cursor: io::Cursor<Vec<u8>>,
}

impl BlockWriter {
    fn new(block: Vec<u8>) -> BlockWriter {
        BlockWriter { cursor: io::Cursor::new(block), }
    }

    fn new_blank(mut block: Vec<u8>) -> BlockWriter {
        block.clear();
        BlockWriter::new(block)
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
