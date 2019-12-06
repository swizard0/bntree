use super::super::{
    plan,
    super::sketch,
};

#[test]
fn tree17_4() {
    let sketch = sketch::Tree::new(17, 4);
    interpret_script(&sketch, vec![
        Instruction::TreeStart,
        Instruction::BlockStart { level_index: 1, block_index: 0, },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 0, },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 1, },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 2, },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 3, },
        Instruction::BlockFinish { level_index: 1, block_index: 0, },
        Instruction::BlockStart { level_index: 0, block_index: 0, },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 0, },
        Instruction::BlockStart { level_index: 1, block_index: 1, },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 0, },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 1, },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 2, },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 3, },
        Instruction::BlockFinish { level_index: 1, block_index: 1, },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 1, },
        Instruction::BlockStart { level_index: 1, block_index: 2, },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 0, },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 1, },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 2, },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 3, },
        Instruction::BlockFinish { level_index: 1, block_index: 2, },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 2, },
        Instruction::BlockStart { level_index: 1, block_index: 3, },
        Instruction::WriteItem { level_index: 1, block_index: 3, item_index: 0, },
        Instruction::BlockFinish { level_index: 1, block_index: 3, },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 3, },
        Instruction::BlockFinish { level_index: 0, block_index: 0, },
        Instruction::Done,
    ]);
}

#[test]
fn tree17_3() {
    let sketch = sketch::Tree::new(17, 3);
    interpret_script(&sketch, vec![
        Instruction::TreeStart,
        Instruction::BlockStart { level_index: 2, block_index: 0 },
        Instruction::WriteItem { level_index: 2, block_index: 0, item_index: 0 },
        Instruction::WriteItem { level_index: 2, block_index: 0, item_index: 1 },
        Instruction::WriteItem { level_index: 2, block_index: 0, item_index: 2 },
        Instruction::BlockFinish { level_index: 2, block_index: 0 },
        Instruction::BlockStart { level_index: 1, block_index: 0 },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 0 },
        Instruction::BlockStart { level_index: 2, block_index: 1 },
        Instruction::WriteItem { level_index: 2, block_index: 1, item_index: 0 },
        Instruction::WriteItem { level_index: 2, block_index: 1, item_index: 1 },
        Instruction::BlockFinish { level_index: 2, block_index: 1 },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 1 },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 2 },
        Instruction::BlockFinish { level_index: 1, block_index: 0 },
        Instruction::BlockStart { level_index: 0, block_index: 0 },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 0 },
        Instruction::BlockStart { level_index: 1, block_index: 1 },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 0 },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 1 },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 2 },
        Instruction::BlockFinish { level_index: 1, block_index: 1 },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 1 },
        Instruction::BlockStart { level_index: 1, block_index: 2 },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 0 },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 1 },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 2 },
        Instruction::BlockFinish { level_index: 1, block_index: 2 },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 2 },
        Instruction::BlockFinish { level_index: 0, block_index: 0 },
        Instruction::Done,
    ]);
}

#[derive(PartialEq, Debug)]
enum Instruction {
    TreeStart,
    BlockStart { level_index: usize, block_index: usize, },
    WriteItem { level_index: usize, block_index: usize, item_index: usize, },
    BlockFinish { level_index: usize, block_index: usize, },
    Done,
}

fn interpret_script(sketch: &sketch::Tree, mut script: Vec<Instruction>) {
    script.reverse();

    let mut instr = plan::Script::start();
    loop {
        use plan::{Perform, Op, PerformBlock, BlockOp};

        match instr {
            plan::Instruction::Perform(Perform { op: Op::TreeStart, next, }) => {
                assert_eq!(script.pop(), Some(Instruction::TreeStart));
                instr = next.step(sketch);
            },
            plan::Instruction::Perform(Perform { op: Op::Block(PerformBlock { op: BlockOp::Start, level_index, block_index, }), next, }) => {
                assert_eq!(script.pop(), Some(Instruction::BlockStart { level_index, block_index, }));
                instr = next.step(sketch);
            },
            plan::Instruction::Perform(
                Perform { op: Op::Block(PerformBlock { op: BlockOp::Item { index: item_index, }, level_index, block_index, }), next, },
            ) => {
                assert_eq!(script.pop(), Some(Instruction::WriteItem { level_index, block_index, item_index, }));
                instr = next.step(sketch);
            },
            plan::Instruction::Perform(Perform { op: Op::Block(PerformBlock { op: BlockOp::Finish, level_index, block_index, }), next, }) => {
                assert_eq!(script.pop(), Some(Instruction::BlockFinish { level_index, block_index, }));
                instr = next.step(sketch);
            },
            plan::Instruction::Done => {
                assert_eq!(script.pop(), Some(Instruction::Done));
                break
            },
        }
    }
}
