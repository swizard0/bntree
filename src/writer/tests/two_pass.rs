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
        let mut fold_ctx = fold::Context::new(sketch);
        let mut markup_ctx = two_pass::markup::Context::new();

        let plan_op = plan::Script::new()
            .step(sketch);
        let fold_op = fold::Script::new()
            .step(&mut fold_ctx, plan_op).unwrap();
        let mut markup_op = two_pass::markup::Script::new()
            .step(&mut markup_ctx, fold_op).unwrap();
        loop {
            let kont = match markup_op {
                two_pass::markup::Instruction::Op(two_pass::markup::Op::InitialLevelSize(
                    two_pass::markup::InitialLevelSize { level_index, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::InitialLevelSize { level_index, }));
                    next.level_header_size(5, &mut fold_ctx).unwrap()
                },
                two_pass::markup::Instruction::Op(two_pass::markup::Op::AllocBlock(
                    two_pass::markup::AllocBlock { level_index, block_index, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::AllocMarkupBlock { level_index, block_index, }));
                    let index = blocks_counter;
                    blocks_counter += 1;
                    next.block_ready(AllocBlock { index, items: 0, }, &mut fold_ctx).unwrap()
                },
                two_pass::markup::Instruction::Op(two_pass::markup::Op::WriteItem(
                    two_pass::markup::WriteItem { level_index, block_index, block_item_index, block, child_pending, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::WriteMarkupItem { level_index, block_index, block_item_index, block, child_pending, }));
                    next.item_written(AllocBlock { items: block.items + 1, ..block }, &mut markup_ctx, &mut fold_ctx).unwrap()
                },
                two_pass::markup::Instruction::Op(two_pass::markup::Op::FinishBlock(
                    two_pass::markup::FinishBlock { level_index, block_index, block, next, },
                )) => {
                    assert_eq!(script.pop(), Some(Instr::FinishMarkupBlock { level_index, block_index, block, }));
                    next.block_finished(block.items, &mut markup_ctx, &mut fold_ctx).unwrap()
                },
                two_pass::markup::Instruction::Done(done) => {
                    assert_eq!(script.pop(), Some(Instr::Done(done.finish(fold_ctx).collect())));
                    break;
                },
            };
            let fold_op = match kont.fold_action {
                two_pass::markup::FoldAction::Idle(fold_op) =>
                    fold_op,
                two_pass::markup::FoldAction::Step(fold::Continue { plan_action: fold::PlanAction::Idle(plan_op), next, }) =>
                    next.step(&mut fold_ctx, plan_op).unwrap(),
                two_pass::markup::FoldAction::Step(fold::Continue { plan_action: fold::PlanAction::Step(plan_script), next, }) => {
                    let plan_op = plan_script.step(sketch);
                    next.step(&mut fold_ctx, plan_op).unwrap()
                },
            };
            markup_op = kont.next.step(&mut markup_ctx, fold_op).unwrap();
        }
    }
}
