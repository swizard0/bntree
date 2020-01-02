use super::super::{
    two_pass,
    fold,
    plan,
    super::sketch,
};

#[test]
fn tree17_4_markup() {
    let sketch = sketch::Tree::new(17, 4);

    use markup::{Instr, AllocBlock};
    markup::interpret(&sketch, vec![
        Instr::InitialLevelSize { level_index: 1 },
        Instr::AllocMarkupBlock { level_index: 1, block_index: 0 },
        Instr::WriteMarkupItem { level_index: 1, block_index: 0, block_item_index: 0, block: AllocBlock { index: 0, items: 0 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 0, block_item_index: 1, block: AllocBlock { index: 0, items: 1 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 0, block_item_index: 2, block: AllocBlock { index: 0, items: 2 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 0, block_item_index: 3, block: AllocBlock { index: 0, items: 3 }, child_pending: false },
        Instr::FinishMarkupBlock { level_index: 1, block_index: 0, block: AllocBlock { index: 0, items: 4 } },
        Instr::InitialLevelSize { level_index: 0 },
        Instr::AllocMarkupBlock { level_index: 0, block_index: 0 },
        Instr::WriteMarkupItem { level_index: 0, block_index: 0, block_item_index: 0, block: AllocBlock { index: 1, items: 0 }, child_pending: true },
        Instr::AllocMarkupBlock { level_index: 1, block_index: 1 },
        Instr::WriteMarkupItem { level_index: 1, block_index: 1, block_item_index: 0, block: AllocBlock { index: 2, items: 0 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 1, block_item_index: 1, block: AllocBlock { index: 2, items: 1 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 1, block_item_index: 2, block: AllocBlock { index: 2, items: 2 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 1, block_item_index: 3, block: AllocBlock { index: 2, items: 3 }, child_pending: false },
        Instr::FinishMarkupBlock { level_index: 1, block_index: 1, block: AllocBlock { index: 2, items: 4 } },
        Instr::WriteMarkupItem { level_index: 0, block_index: 0, block_item_index: 1, block: AllocBlock { index: 1, items: 1 }, child_pending: true },
        Instr::AllocMarkupBlock { level_index: 1, block_index: 2 },
        Instr::WriteMarkupItem { level_index: 1, block_index: 2, block_item_index: 0, block: AllocBlock { index: 3, items: 0 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 2, block_item_index: 1, block: AllocBlock { index: 3, items: 1 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 2, block_item_index: 2, block: AllocBlock { index: 3, items: 2 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 2, block_item_index: 3, block: AllocBlock { index: 3, items: 3 }, child_pending: false },
        Instr::FinishMarkupBlock { level_index: 1, block_index: 2, block: AllocBlock { index: 3, items: 4 } },
        Instr::WriteMarkupItem { level_index: 0, block_index: 0, block_item_index: 2, block: AllocBlock { index: 1, items: 2 }, child_pending: true },
        Instr::AllocMarkupBlock { level_index: 1, block_index: 3 },
        Instr::WriteMarkupItem { level_index: 1, block_index: 3, block_item_index: 0, block: AllocBlock { index: 4, items: 0 }, child_pending: false },
        Instr::FinishMarkupBlock { level_index: 1, block_index: 3, block: AllocBlock { index: 4, items: 1 } },
        Instr::WriteMarkupItem { level_index: 0, block_index: 0, block_item_index: 3, block: AllocBlock { index: 1, items: 3 }, child_pending: true },
        Instr::FinishMarkupBlock { level_index: 0, block_index: 0, block: AllocBlock { index: 1, items: 4 } },
        Instr::Done(vec![
            two_pass::LevelCoords { index: 0, header_size: 5, total_size: 9 },
            two_pass::LevelCoords { index: 1, header_size: 5, total_size: 18 },
        ]),
    ]);
}

#[test]
fn tree17_3_markup() {
    let sketch = sketch::Tree::new(17, 3);

    use markup::{Instr, AllocBlock};
    markup::interpret(&sketch, vec![
        Instr::InitialLevelSize { level_index: 2 },
        Instr::AllocMarkupBlock { level_index: 2, block_index: 0 },
        Instr::WriteMarkupItem { level_index: 2, block_index: 0, block_item_index: 0, block: AllocBlock { index: 0, items: 0 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 2, block_index: 0, block_item_index: 1, block: AllocBlock { index: 0, items: 1 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 2, block_index: 0, block_item_index: 2, block: AllocBlock { index: 0, items: 2 }, child_pending: false },
        Instr::FinishMarkupBlock { level_index: 2, block_index: 0, block: AllocBlock { index: 0, items: 3 } },
        Instr::InitialLevelSize { level_index: 1 },
        Instr::AllocMarkupBlock { level_index: 1, block_index: 0 },
        Instr::WriteMarkupItem { level_index: 1, block_index: 0, block_item_index: 0, block: AllocBlock { index: 1, items: 0 }, child_pending: true },
        Instr::AllocMarkupBlock { level_index: 2, block_index: 1 },
        Instr::WriteMarkupItem { level_index: 2, block_index: 1, block_item_index: 0, block: AllocBlock { index: 2, items: 0 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 2, block_index: 1, block_item_index: 1, block: AllocBlock { index: 2, items: 1 }, child_pending: false },
        Instr::FinishMarkupBlock { level_index: 2, block_index: 1, block: AllocBlock { index: 2, items: 2 } },
        Instr::WriteMarkupItem { level_index: 1, block_index: 0, block_item_index: 1, block: AllocBlock { index: 1, items: 1 }, child_pending: true },
        Instr::WriteMarkupItem { level_index: 1, block_index: 0, block_item_index: 2, block: AllocBlock { index: 1, items: 2 }, child_pending: false },
        Instr::FinishMarkupBlock { level_index: 1, block_index: 0, block: AllocBlock { index: 1, items: 3 } },
        Instr::InitialLevelSize { level_index: 0 },
        Instr::AllocMarkupBlock { level_index: 0, block_index: 0 },
        Instr::WriteMarkupItem { level_index: 0, block_index: 0, block_item_index: 0, block: AllocBlock { index: 3, items: 0 }, child_pending: true },
        Instr::AllocMarkupBlock { level_index: 1, block_index: 1 },
        Instr::WriteMarkupItem { level_index: 1, block_index: 1, block_item_index: 0, block: AllocBlock { index: 4, items: 0 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 1, block_item_index: 1, block: AllocBlock { index: 4, items: 1 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 1, block_item_index: 2, block: AllocBlock { index: 4, items: 2 }, child_pending: false },
        Instr::FinishMarkupBlock { level_index: 1, block_index: 1, block: AllocBlock { index: 4, items: 3 } },
        Instr::WriteMarkupItem { level_index: 0, block_index: 0, block_item_index: 1, block: AllocBlock { index: 3, items: 1 }, child_pending: true },
        Instr::AllocMarkupBlock { level_index: 1, block_index: 2 },
        Instr::WriteMarkupItem { level_index: 1, block_index: 2, block_item_index: 0, block: AllocBlock { index: 5, items: 0 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 2, block_item_index: 1, block: AllocBlock { index: 5, items: 1 }, child_pending: false },
        Instr::WriteMarkupItem { level_index: 1, block_index: 2, block_item_index: 2, block: AllocBlock { index: 5, items: 2 }, child_pending: false },
        Instr::FinishMarkupBlock { level_index: 1, block_index: 2, block: AllocBlock { index: 5, items: 3 } },
        Instr::WriteMarkupItem { level_index: 0, block_index: 0, block_item_index: 2, block: AllocBlock { index: 3, items: 2 }, child_pending: true },
        Instr::FinishMarkupBlock { level_index: 0, block_index: 0, block: AllocBlock { index: 3, items: 3 } },
        Instr::Done(vec![
            two_pass::LevelCoords { index: 0, header_size: 5, total_size: 8 },
            two_pass::LevelCoords { index: 1, header_size: 5, total_size: 14 },
            two_pass::LevelCoords { index: 2, header_size: 5, total_size: 10 }
        ]),
    ]);
}

#[test]
fn tree17_4_write() {
    let sketch = sketch::Tree::new(17, 4);

    let levels_coords = vec![
        two_pass::LevelCoords { index: 0, header_size: 5, total_size: 9 },
        two_pass::LevelCoords { index: 1, header_size: 5, total_size: 18 },
    ];
    use write::Instr;
    write::interpret(&sketch, levels_coords.into_iter(), vec![
        Instr::WriteTreeHeader { tree_offset: 1000, tree_header_size: 3, tree_total_size: 30 },
        Instr::WriteLevelHeader { level_index: 1, level_offset: 1012 },
        Instr::WriteBlockHeader { level_index: 1, block_index: 0, block_offset: 1017 },
        Instr::WriteItem { level_index: 1, block_index: 0, block_item_index: 0, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 0, block_item_index: 1, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 0, block_item_index: 2, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 0, block_item_index: 3, child_block_offset: None },
        Instr::FlushBlock { level_index: 1, block_index: 0, block_start_offset: 1017, block_end_offset: 1021 },
        Instr::WriteLevelHeader { level_index: 0, level_offset: 1003 },
        Instr::WriteBlockHeader { level_index: 0, block_index: 0, block_offset: 1008 },
        Instr::WriteItem { level_index: 0, block_index: 0, block_item_index: 0, child_block_offset: Some(1017) },
        Instr::WriteBlockHeader { level_index: 1, block_index: 1, block_offset: 1021 },
        Instr::WriteItem { level_index: 1, block_index: 1, block_item_index: 0, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 1, block_item_index: 1, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 1, block_item_index: 2, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 1, block_item_index: 3, child_block_offset: None },
        Instr::FlushBlock { level_index: 1, block_index: 1, block_start_offset: 1021, block_end_offset: 1025 },
        Instr::WriteItem { level_index: 0, block_index: 0, block_item_index: 1, child_block_offset: Some(1021) },
        Instr::WriteBlockHeader { level_index: 1, block_index: 2, block_offset: 1025 },
        Instr::WriteItem { level_index: 1, block_index: 2, block_item_index: 0, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 2, block_item_index: 1, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 2, block_item_index: 2, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 2, block_item_index: 3, child_block_offset: None },
        Instr::FlushBlock { level_index: 1, block_index: 2, block_start_offset: 1025, block_end_offset: 1029 },
        Instr::WriteItem { level_index: 0, block_index: 0, block_item_index: 2, child_block_offset: Some(1025) },
        Instr::WriteBlockHeader { level_index: 1, block_index: 3, block_offset: 1029 },
        Instr::WriteItem { level_index: 1, block_index: 3, block_item_index: 0, child_block_offset: None },
        Instr::FlushBlock { level_index: 1, block_index: 3, block_start_offset: 1029, block_end_offset: 1030 },
        Instr::WriteItem { level_index: 0, block_index: 0, block_item_index: 3, child_block_offset: Some(1029) },
        Instr::FlushBlock { level_index: 0, block_index: 0, block_start_offset: 1008, block_end_offset: 1012 },
        Instr::Done,
    ]);
}

#[test]
fn tree17_3_write() {
    let sketch = sketch::Tree::new(17, 3);

    let levels_coords = vec![
        two_pass::LevelCoords { index: 0, header_size: 5, total_size: 8 },
        two_pass::LevelCoords { index: 1, header_size: 5, total_size: 14 },
        two_pass::LevelCoords { index: 2, header_size: 5, total_size: 10 }
    ];
    use write::Instr;
    write::interpret(&sketch, levels_coords.into_iter(), vec![
        Instr::WriteTreeHeader { tree_offset: 1000, tree_header_size: 3, tree_total_size: 35 },
        Instr::WriteLevelHeader { level_index: 2, level_offset: 1025 },
        Instr::WriteBlockHeader { level_index: 2, block_index: 0, block_offset: 1030 },
        Instr::WriteItem { level_index: 2, block_index: 0, block_item_index: 0, child_block_offset: None },
        Instr::WriteItem { level_index: 2, block_index: 0, block_item_index: 1, child_block_offset: None },
        Instr::WriteItem { level_index: 2, block_index: 0, block_item_index: 2, child_block_offset: None },
        Instr::FlushBlock { level_index: 2, block_index: 0, block_start_offset: 1030, block_end_offset: 1033 },
        Instr::WriteLevelHeader { level_index: 1, level_offset: 1011 },
        Instr::WriteBlockHeader { level_index: 1, block_index: 0, block_offset: 1016 },
        Instr::WriteItem { level_index: 1, block_index: 0, block_item_index: 0, child_block_offset: Some(1030) },
        Instr::WriteBlockHeader { level_index: 2, block_index: 1, block_offset: 1033 },
        Instr::WriteItem { level_index: 2, block_index: 1, block_item_index: 0, child_block_offset: None },
        Instr::WriteItem { level_index: 2, block_index: 1, block_item_index: 1, child_block_offset: None },
        Instr::FlushBlock { level_index: 2, block_index: 1, block_start_offset: 1033, block_end_offset: 1035 },
        Instr::WriteItem { level_index: 1, block_index: 0, block_item_index: 1, child_block_offset: Some(1033) },
        Instr::WriteItem { level_index: 1, block_index: 0, block_item_index: 2, child_block_offset: None },
        Instr::FlushBlock { level_index: 1, block_index: 0, block_start_offset: 1016, block_end_offset: 1019 },
        Instr::WriteLevelHeader { level_index: 0, level_offset: 1003 },
        Instr::WriteBlockHeader { level_index: 0, block_index: 0, block_offset: 1008 },
        Instr::WriteItem { level_index: 0, block_index: 0, block_item_index: 0, child_block_offset: Some(1016) },
        Instr::WriteBlockHeader { level_index: 1, block_index: 1, block_offset: 1019 },
        Instr::WriteItem { level_index: 1, block_index: 1, block_item_index: 0, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 1, block_item_index: 1, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 1, block_item_index: 2, child_block_offset: None },
        Instr::FlushBlock { level_index: 1, block_index: 1, block_start_offset: 1019, block_end_offset: 1022 },
        Instr::WriteItem { level_index: 0, block_index: 0, block_item_index: 1, child_block_offset: Some(1019) },
        Instr::WriteBlockHeader { level_index: 1, block_index: 2, block_offset: 1022 },
        Instr::WriteItem { level_index: 1, block_index: 2, block_item_index: 0, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 2, block_item_index: 1, child_block_offset: None },
        Instr::WriteItem { level_index: 1, block_index: 2, block_item_index: 2, child_block_offset: None },
        Instr::FlushBlock { level_index: 1, block_index: 2, block_start_offset: 1022, block_end_offset: 1025 },
        Instr::WriteItem { level_index: 0, block_index: 0, block_item_index: 2, child_block_offset: Some(1022) },
        Instr::FlushBlock { level_index: 0, block_index: 0, block_start_offset: 1008, block_end_offset: 1011 },
        Instr::Done,
    ]);
}


mod markup {
    use super::{sketch, plan, fold, two_pass};

    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct AllocBlock {
        pub index: usize,
        pub items: usize,
    }

    #[derive(Clone, PartialEq, Debug)]
    pub enum Instr {
        InitialLevelSize { level_index: usize, },
        AllocMarkupBlock { level_index: usize, block_index: usize, },
        WriteMarkupItem { level_index: usize, block_index: usize, block_item_index: usize, block: AllocBlock, child_pending: bool, },
        FinishMarkupBlock { level_index: usize, block_index: usize,  block: AllocBlock, },
        Done(Vec<two_pass::LevelCoords<usize>>),
    }

    pub fn interpret(sketch: &sketch::Tree, mut script: Vec<Instr>) {
        script.reverse();

        let mut blocks_counter = 0;
        let mut markup_ctx = two_pass::markup::Context::new(
            fold::Context::new(plan::Context::new(sketch), sketch),
        );

        let mut kont = two_pass::markup::Script::boot();
        loop {
            kont = match kont.step_rec(&mut markup_ctx).unwrap() {
                two_pass::markup::Instruction::Op(two_pass::markup::Op::LevelHeaderSize(
                    two_pass::markup::LevelHeaderSize { level_index, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::InitialLevelSize { level_index, }));
                    next.level_header_size(5, &mut markup_ctx).unwrap()
                },
                two_pass::markup::Instruction::Op(two_pass::markup::Op::AllocBlock(
                    two_pass::markup::AllocBlock { level_index, block_index, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::AllocMarkupBlock { level_index, block_index, }));
                    let index = blocks_counter;
                    blocks_counter += 1;
                    next.block_ready(AllocBlock { index, items: 0, }, &mut markup_ctx).unwrap()
                },
                two_pass::markup::Instruction::Op(two_pass::markup::Op::WriteItem(
                    two_pass::markup::WriteItem { level_index, block_index, block_item_index, block, child_pending, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::WriteMarkupItem { level_index, block_index, block_item_index, block, child_pending, }));
                    next.item_written(AllocBlock { items: block.items + 1, ..block }, &mut markup_ctx).unwrap()
                },
                two_pass::markup::Instruction::Op(two_pass::markup::Op::FinishBlock(
                    two_pass::markup::FinishBlock { level_index, block_index, block, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::FinishMarkupBlock { level_index, block_index, block, }));
                    next.block_finished(block.items, &mut markup_ctx).unwrap()
                },
                two_pass::markup::Instruction::Done => {
                    assert_eq!(script.pop(), Some(Instr::Done(
                        markup_ctx.into_level_coords_iter()
                            .collect(),
                    )));
                    break;
                },
            };
        }
    }
}

mod write {
    use super::{sketch, plan, fold, two_pass};

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct BlockMeta;

    #[derive(Clone, PartialEq, Debug)]
    pub enum Instr {
        WriteTreeHeader { tree_offset: usize, tree_header_size: usize, tree_total_size: usize, },
        WriteLevelHeader { level_index: usize, level_offset: usize, },
        WriteBlockHeader { level_index: usize, block_index: usize, block_offset: usize, },
        WriteItem { level_index: usize, block_index: usize, block_item_index: usize, child_block_offset: Option<usize>, },
        FlushBlock { level_index: usize, block_index: usize, block_start_offset: usize, block_end_offset: usize, },
        Done,
    }

    pub fn interpret<I>(sketch: &sketch::Tree, levels_coords: I, mut script: Vec<Instr>) where I: Iterator<Item = two_pass::LevelCoords<usize>>{
        script.reverse();

        let mut write_ctx =
            two_pass::write::Context::new(
                fold::Context::new(plan::Context::new(sketch), sketch),
                1000,
                3,
                levels_coords,
            ).unwrap();

        let mut kont = two_pass::write::Script::boot();
        loop {
            kont = match kont.step_rec(&mut write_ctx).unwrap() {
                two_pass::write::Instruction::Op(two_pass::write::Op::WriteTreeHeader(
                    two_pass::write::WriteTreeHeader { tree_offset, tree_header_size, tree_total_size, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::WriteTreeHeader { tree_offset, tree_header_size, tree_total_size, }));
                    next.tree_header_written(3, &mut write_ctx).unwrap()
                },
                two_pass::write::Instruction::Op(two_pass::write::Op::WriteLevelHeader(
                    two_pass::write::WriteLevelHeader { level_index, level_offset, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::WriteLevelHeader { level_index, level_offset, }));
                    next.level_header_written(5, &mut write_ctx).unwrap()
                },
                two_pass::write::Instruction::Op(two_pass::write::Op::WriteBlockHeader(
                    two_pass::write::WriteBlockHeader { level_index, block_index, block_offset, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::WriteBlockHeader { level_index, block_index, block_offset, }));
                    next.block_header_written(BlockMeta, 0, &mut write_ctx).unwrap()
                },
                two_pass::write::Instruction::Op(two_pass::write::Op::WriteItem(
                    two_pass::write::WriteItem { level_index, block_index, block_item_index, block_meta: BlockMeta, child_block_offset, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::WriteItem { level_index, block_index, block_item_index, child_block_offset, }));
                    next.item_written(BlockMeta, 1, &mut write_ctx).unwrap()
                },
                two_pass::write::Instruction::Op(two_pass::write::Op::FlushBlock(
                    two_pass::write::FlushBlock { level_index, block_index, block_meta: BlockMeta, block_start_offset, block_end_offset, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::FlushBlock { level_index, block_index, block_start_offset, block_end_offset, }));
                    next.block_flushed(block_end_offset - block_start_offset, &mut write_ctx).unwrap()
                },
                two_pass::write::Instruction::Done => {
                    assert_eq!(script.pop(), Some(Instr::Done));
                    break;
                },
            };
        }
    }
}
