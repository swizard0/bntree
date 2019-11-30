use super::{
    sketch,
    plan,
    fold,
    two_pass,
};

#[test]
fn tree17_4() {
    let sketch = sketch::Tree::new(17, 4);
    assert_eq!(
        sketch.levels(),
        &[
            sketch::Level { index: 0, blocks_count: 1, items_count: 4 },
            sketch::Level { index: 1, blocks_count: 4, items_count: 13 },
        ]
    );

    let script: Vec<_> = plan::build(&sketch).collect();
    assert_eq!(
        script,
        vec![
            plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::BlockStart },
            plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 0 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 1 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 2 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 3 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::BlockFinish },
            plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::BlockStart },
            plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 0 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::BlockStart },
            plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 0 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 1 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 2 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 3 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::BlockFinish },
            plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 1 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::BlockStart },
            plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 0 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 1 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 2 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 3 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::BlockFinish },
            plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 2 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 3, op: plan::Op::BlockStart },
            plan::Instruction { level: &sketch.levels()[1], block_index: 3, op: plan::Op::WriteItem { block_item_index: 0 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 3, op: plan::Op::BlockFinish },
            plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 3 } },
            plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::BlockFinish },
        ],
    );

    assert_eq!(
        fold_identity(&sketch),
        vec![
            (&sketch.levels()[0], (sketch.levels()[0].items_count, sketch.levels()[0].blocks_count)),
            (&sketch.levels()[1], (sketch.levels()[1].items_count, sketch.levels()[1].blocks_count)),
        ],
    );
}

#[test]
fn tree17_4_two_pass() {
    let sketch = sketch::Tree::new(17, 4);

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct AllocBlock {
        index: usize,
        items: usize,
    }

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct BlockMeta;

    #[derive(PartialEq, Debug)]
    enum ExpectedInstr {
        InitialLevelSize { level_index: usize, },
        AllocMarkupBlock { level_index: usize, },
        WriteMarkupItem { level_index: usize, block: AllocBlock, },
        FinishMarkupBlock { level_index: usize, block: AllocBlock, },
        WriteTreeHeader { offset: usize, tree_total_size: usize, },
        WriteLevelHeader { level_index: usize, level_offset: usize, },
        WriteBlockHeader { level_index: usize, block_offset: usize, },
        WriteBlockItem { level_index: usize, block_meta: BlockMeta, item_offset: usize, child_block_offset: Option<usize>, },
        FlushBlock { level_index: usize, block_meta: BlockMeta, block_start_offset: usize, block_end_offset: usize, },
    }

    let mut script = vec![
        ExpectedInstr::InitialLevelSize { level_index: 1, },
        ExpectedInstr::AllocMarkupBlock { level_index: 1, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 0, items: 0, }, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 0, items: 1, }, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 0, items: 2, }, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 0, items: 3, }, },
        ExpectedInstr::FinishMarkupBlock { level_index: 1, block: AllocBlock { index: 0, items: 4, }, },
        ExpectedInstr::InitialLevelSize { level_index: 0, },
        ExpectedInstr::AllocMarkupBlock { level_index: 0, },
        ExpectedInstr::WriteMarkupItem { level_index: 0, block: AllocBlock { index: 1, items: 0, }, },
        ExpectedInstr::AllocMarkupBlock { level_index: 1, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 2, items: 0, }, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 2, items: 1, }, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 2, items: 2, }, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 2, items: 3, }, },
        ExpectedInstr::FinishMarkupBlock { level_index: 1, block: AllocBlock { index: 2, items: 4, }, },
        ExpectedInstr::WriteMarkupItem { level_index: 0, block: AllocBlock { index: 1, items: 1, }, },
        ExpectedInstr::AllocMarkupBlock { level_index: 1, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 3, items: 0, }, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 3, items: 1, }, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 3, items: 2, }, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 3, items: 3, }, },
        ExpectedInstr::FinishMarkupBlock { level_index: 1, block: AllocBlock { index: 3, items: 4, }, },
        ExpectedInstr::WriteMarkupItem { level_index: 0, block: AllocBlock { index: 1, items: 2, }, },
        ExpectedInstr::AllocMarkupBlock { level_index: 1, },
        ExpectedInstr::WriteMarkupItem { level_index: 1, block: AllocBlock { index: 4, items: 0, }, },
        ExpectedInstr::FinishMarkupBlock { level_index: 1, block: AllocBlock { index: 4, items: 1, }, },
        ExpectedInstr::WriteMarkupItem { level_index: 0, block: AllocBlock { index: 1, items: 3, }, },
        ExpectedInstr::FinishMarkupBlock { level_index: 0, block: AllocBlock { index: 1, items: 4, }, },

        ExpectedInstr::WriteTreeHeader { offset: 0, tree_total_size: 30, },
        ExpectedInstr::WriteLevelHeader { level_index: 1, level_offset: 12, },
        ExpectedInstr::WriteBlockHeader { level_index: 1, block_offset: 17, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 17, child_block_offset: None, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 18, child_block_offset: None, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 19, child_block_offset: None, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 20, child_block_offset: None, },
        ExpectedInstr::FlushBlock { level_index: 1, block_meta: BlockMeta, block_start_offset: 17, block_end_offset: 21, },
        ExpectedInstr::WriteLevelHeader { level_index: 0, level_offset: 3, },
        ExpectedInstr::WriteBlockHeader { level_index: 0, block_offset: 8, },
        ExpectedInstr::WriteBlockItem { level_index: 0, block_meta: BlockMeta, item_offset: 8, child_block_offset: Some(17), },
        ExpectedInstr::WriteBlockHeader { level_index: 1, block_offset: 21, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 21, child_block_offset: None, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 22, child_block_offset: None, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 23, child_block_offset: None, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 24, child_block_offset: None, },
        ExpectedInstr::FlushBlock { level_index: 1, block_meta: BlockMeta, block_start_offset: 21, block_end_offset: 25, },
        ExpectedInstr::WriteBlockItem { level_index: 0, block_meta: BlockMeta, item_offset: 9, child_block_offset: Some(21), },
        ExpectedInstr::WriteBlockHeader { level_index: 1, block_offset: 25, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 25, child_block_offset: None, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 26, child_block_offset: None, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 27, child_block_offset: None, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 28, child_block_offset: None, },
        ExpectedInstr::FlushBlock { level_index: 1, block_meta: BlockMeta, block_start_offset: 25, block_end_offset: 29, },
        ExpectedInstr::WriteBlockItem { level_index: 0, block_meta: BlockMeta, item_offset: 10, child_block_offset: Some(25), },
        ExpectedInstr::WriteBlockHeader { level_index: 1, block_offset: 29, },
        ExpectedInstr::WriteBlockItem { level_index: 1, block_meta: BlockMeta, item_offset: 29, child_block_offset: None, },
        ExpectedInstr::FlushBlock { level_index: 1, block_meta: BlockMeta, block_start_offset: 29, block_end_offset: 30, },
        ExpectedInstr::WriteBlockItem { level_index: 0, block_meta: BlockMeta, item_offset: 11, child_block_offset: Some(29), },
        ExpectedInstr::FlushBlock { level_index: 0, block_meta: BlockMeta, block_start_offset: 8, block_end_offset: 12, },
    ];
    script.reverse();

    let mut blocks_counter = 0;
    let mut write_blocks = two_pass::write_blocks(&sketch, 0, 3);
    loop {
        match write_blocks.next() {
            two_pass::Instruction::InitialLevelSize(two_pass::InitialLevelSize { level, next, }) => {
                assert_eq!(script.pop(), Some(ExpectedInstr::InitialLevelSize { level_index: level.index, }));
                let two_pass::AllocMarkupBlock { level, next, } =
                    next.level_header_size(5);
                assert_eq!(script.pop(), Some(ExpectedInstr::AllocMarkupBlock { level_index: level.index, }));
                write_blocks = next.block_ready(AllocBlock { index: blocks_counter, items: 0, });
                blocks_counter += 1;
            },
            two_pass::Instruction::AllocMarkupBlock(two_pass::AllocMarkupBlock { level, next, }) => {
                assert_eq!(script.pop(), Some(ExpectedInstr::AllocMarkupBlock { level_index: level.index, }));
                write_blocks = next.block_ready(AllocBlock { index: blocks_counter, items: 0, });
                blocks_counter += 1;
            },
            two_pass::Instruction::WriteMarkupItem(two_pass::WriteMarkupItem { level, block, next, }) => {
                assert_eq!(script.pop(), Some(ExpectedInstr::WriteMarkupItem { level_index: level.index, block, }));
                write_blocks = next.item_written(AllocBlock { items: block.items + 1, ..block });
            },
            two_pass::Instruction::FinishMarkupBlock(two_pass::FinishMarkupBlock { level, block, next, }) => {
                assert_eq!(script.pop(), Some(ExpectedInstr::FinishMarkupBlock { level_index: level.index, block, }));
                write_blocks = next.block_finished(block.items);
            },
            two_pass::Instruction::WriteTreeHeader(two_pass::WriteTreeHeader { offset, tree_total_size, next, }) => {
                assert_eq!(script.pop(), Some(ExpectedInstr::WriteTreeHeader { offset, tree_total_size, }));
                write_blocks = next.tree_header_written(3);
            },
            two_pass::Instruction::WriteLevelHeader(two_pass::WriteLevelHeader { level, level_offset, next, }) => {
                assert_eq!(script.pop(), Some(ExpectedInstr::WriteLevelHeader { level_index: level.index, level_offset, }));
                let two_pass::WriteBlockHeader { level, block_offset, next, } =
                    next.level_header_written(5);
                assert_eq!(script.pop(), Some(ExpectedInstr::WriteBlockHeader { level_index: level.index, block_offset, }));
                write_blocks = next.block_header_written(BlockMeta, 0);
            },
            two_pass::Instruction::WriteBlockHeader(two_pass::WriteBlockHeader { level, block_offset, next, }) => {
                assert_eq!(script.pop(), Some(ExpectedInstr::WriteBlockHeader { level_index: level.index, block_offset, }));
                write_blocks = next.block_header_written(BlockMeta, 0);
            },
            two_pass::Instruction::WriteBlockItem(two_pass::WriteBlockItem { level, block_meta, item_offset, child_block_offset, next, }) => {
                assert_eq!(
                    script.pop(),
                    Some(ExpectedInstr::WriteBlockItem {
                        level_index: level.index,
                        block_meta,
                        item_offset,
                        child_block_offset,
                    }),
                );
                write_blocks = next.item_written(block_meta, 1);
            },
            two_pass::Instruction::FlushBlock(two_pass::FlushBlock { level, block_meta, block_start_offset, block_end_offset, next, }) => {
                assert_eq!(
                    script.pop(),
                    Some(ExpectedInstr::FlushBlock {
                        level_index: level.index,
                        block_meta,
                        block_start_offset,
                        block_end_offset,
                    }),
                );
                write_blocks = next.block_flushed();
            },
            two_pass::Instruction::Done =>
                break,
        }
    }
}

#[test]
fn tree17_3() {
    let sketch = sketch::Tree::new(17, 3);
    assert_eq!(
        sketch.levels(),
        &[
            sketch::Level { index: 0, blocks_count: 1, items_count: 3 },
            sketch::Level { index: 1, blocks_count: 3, items_count: 9 },
            sketch::Level { index: 2, blocks_count: 2, items_count: 5 },
        ]
    );

    let script: Vec<_> = plan::build(&sketch).collect();
    assert_eq!(
        script,
        vec![
            plan::Instruction { level: &sketch.levels()[2], block_index: 0, op: plan::Op::BlockStart },
            plan::Instruction { level: &sketch.levels()[2], block_index: 0, op: plan::Op::WriteItem { block_item_index: 0 } },
            plan::Instruction { level: &sketch.levels()[2], block_index: 0, op: plan::Op::WriteItem { block_item_index: 1 } },
            plan::Instruction { level: &sketch.levels()[2], block_index: 0, op: plan::Op::WriteItem { block_item_index: 2 } },
            plan::Instruction { level: &sketch.levels()[2], block_index: 0, op: plan::Op::BlockFinish },
            plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::BlockStart },
            plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 0 } },
            plan::Instruction { level: &sketch.levels()[2], block_index: 1, op: plan::Op::BlockStart },
            plan::Instruction { level: &sketch.levels()[2], block_index: 1, op: plan::Op::WriteItem { block_item_index: 0 } },
            plan::Instruction { level: &sketch.levels()[2], block_index: 1, op: plan::Op::WriteItem { block_item_index: 1 } },
            plan::Instruction { level: &sketch.levels()[2], block_index: 1, op: plan::Op::BlockFinish },
            plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 1 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::WriteItem { block_item_index: 2 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 0, op: plan::Op::BlockFinish },
            plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::BlockStart },
            plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 0 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::BlockStart },
            plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 0 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 1 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::WriteItem { block_item_index: 2 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 1, op: plan::Op::BlockFinish },
            plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 1 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::BlockStart },
            plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 0 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 1 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::WriteItem { block_item_index: 2 } },
            plan::Instruction { level: &sketch.levels()[1], block_index: 2, op: plan::Op::BlockFinish },
            plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::WriteItem { block_item_index: 2 } },
            plan::Instruction { level: &sketch.levels()[0], block_index: 0, op: plan::Op::BlockFinish },
        ],
    );

    assert_eq!(
        fold_identity(&sketch),
        vec![
            (&sketch.levels()[0], (sketch.levels()[0].items_count, sketch.levels()[0].blocks_count)),
            (&sketch.levels()[1], (sketch.levels()[1].items_count, sketch.levels()[1].blocks_count)),
            (&sketch.levels()[2], (sketch.levels()[2].items_count, sketch.levels()[2].blocks_count)),
        ],
    );
}

fn fold_identity<'s>(sketch: &'s sketch::Tree) -> Vec<(&'s sketch::Level, (usize, usize))> {
    struct Block;
    let mut fold = fold::fold_levels(sketch);
    loop {
        fold = match fold.next() {
            fold::Instruction::Done(done) =>
                return done.levels_iter().map(|value| (value.0, value.1.clone())).collect(),
            fold::Instruction::VisitLevel(fold::VisitLevel { next, .. }) => {
                let fold::VisitBlockStart { level_seed, next, .. } =
                    next.level_ready((0, 0));
                next.block_ready(Block, level_seed)
            },
            fold::Instruction::VisitBlockStart(fold::VisitBlockStart { level_seed, next, .. }) =>
                next.block_ready(Block, level_seed),
            fold::Instruction::VisitItem(fold::VisitItem { level_seed: (items, blocks), block: Block, next, .. }) =>
                next.item_ready(Block, (items + 1, blocks)),
            fold::Instruction::VisitBlockFinish(fold::VisitBlockFinish { level_seed: (items, blocks), block: Block, next, .. }) =>
                next.block_flushed((items, blocks + 1)),
        }
    }
}
