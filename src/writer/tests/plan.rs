use super::super::{
    plan,
    super::sketch,
};

#[test]
fn tree17_4() {
    let sketch = sketch::Tree::new(17, 4);
    interpret_script(&sketch, vec![
        Instruction::TreeStart,
        Instruction::BlockStart { level_index: 1, block_index: 0, items_count: 4, },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 0, },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 1, },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 2, },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 3, },
        Instruction::BlockFinish { level_index: 1, block_index: 0, },
        Instruction::BlockStart { level_index: 0, block_index: 0, items_count: 4, },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 0, },
        Instruction::BlockStart { level_index: 1, block_index: 1, items_count: 4, },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 0, },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 1, },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 2, },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 3, },
        Instruction::BlockFinish { level_index: 1, block_index: 1, },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 1, },
        Instruction::BlockStart { level_index: 1, block_index: 2, items_count: 4, },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 0, },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 1, },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 2, },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 3, },
        Instruction::BlockFinish { level_index: 1, block_index: 2, },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 2, },
        Instruction::BlockStart { level_index: 1, block_index: 3, items_count: 1, },
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
        Instruction::BlockStart { level_index: 2, block_index: 0, items_count: 3, },
        Instruction::WriteItem { level_index: 2, block_index: 0, item_index: 0 },
        Instruction::WriteItem { level_index: 2, block_index: 0, item_index: 1 },
        Instruction::WriteItem { level_index: 2, block_index: 0, item_index: 2 },
        Instruction::BlockFinish { level_index: 2, block_index: 0 },
        Instruction::BlockStart { level_index: 1, block_index: 0, items_count: 3, },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 0 },
        Instruction::BlockStart { level_index: 2, block_index: 1, items_count: 2, },
        Instruction::WriteItem { level_index: 2, block_index: 1, item_index: 0 },
        Instruction::WriteItem { level_index: 2, block_index: 1, item_index: 1 },
        Instruction::BlockFinish { level_index: 2, block_index: 1 },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 1 },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 2 },
        Instruction::BlockFinish { level_index: 1, block_index: 0 },
        Instruction::BlockStart { level_index: 0, block_index: 0, items_count: 3, },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 0 },
        Instruction::BlockStart { level_index: 1, block_index: 1, items_count: 3, },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 0 },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 1 },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 2 },
        Instruction::BlockFinish { level_index: 1, block_index: 1 },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 1 },
        Instruction::BlockStart { level_index: 1, block_index: 2, items_count: 3, },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 0 },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 1 },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 2 },
        Instruction::BlockFinish { level_index: 1, block_index: 2 },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 2 },
        Instruction::BlockFinish { level_index: 0, block_index: 0 },
        Instruction::Done,
    ]);
}

#[test]
fn tree22_3() {
    let sketch = sketch::Tree::new(22, 3);

    // Level { index: 0, blocks_count: 1, items_count: 3 }
    // Level { index: 1, blocks_count: 3, items_count: 9 }
    // Level { index: 2, blocks_count: 4, items_count: 10 }]

    //                                   (12 17 21)
    //                          (3 7 11)      (14 15 16) (18 19 20)
    // (0 1 2) (4 5 6) (8 9 10)          (13)
    interpret_script(&sketch, vec![
        Instruction::TreeStart,
        Instruction::BlockStart { level_index: 2, block_index: 0, items_count: 3, },
        Instruction::WriteItem { level_index: 2, block_index: 0, item_index: 0 }, // 0
        Instruction::WriteItem { level_index: 2, block_index: 0, item_index: 1 }, // 1
        Instruction::WriteItem { level_index: 2, block_index: 0, item_index: 2 }, // 2
        Instruction::BlockFinish { level_index: 2, block_index: 0 },
        Instruction::BlockStart { level_index: 1, block_index: 0, items_count: 3, },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 0 }, // 3
        Instruction::BlockStart { level_index: 2, block_index: 1, items_count: 3, },
        Instruction::WriteItem { level_index: 2, block_index: 1, item_index: 0 }, // 4
        Instruction::WriteItem { level_index: 2, block_index: 1, item_index: 1 }, // 5
        Instruction::WriteItem { level_index: 2, block_index: 1, item_index: 2 }, // 6
        Instruction::BlockFinish { level_index: 2, block_index: 1 },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 1 }, // 7
        Instruction::BlockStart { level_index: 2, block_index: 2, items_count: 3, },
        Instruction::WriteItem { level_index: 2, block_index: 2, item_index: 0 }, // 8
        Instruction::WriteItem { level_index: 2, block_index: 2, item_index: 1 }, // 9
        Instruction::WriteItem { level_index: 2, block_index: 2, item_index: 2 }, // 10
        Instruction::BlockFinish { level_index: 2, block_index: 2 },
        Instruction::WriteItem { level_index: 1, block_index: 0, item_index: 2 }, // 11
        Instruction::BlockFinish { level_index: 1, block_index: 0 },
        Instruction::BlockStart { level_index: 0, block_index: 0, items_count: 3, },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 0 }, // 12
        Instruction::BlockStart { level_index: 2, block_index: 0, items_count: 1, },
        Instruction::WriteItem { level_index: 2, block_index: 0, item_index: 0 }, // 13
        Instruction::BlockFinish { level_index: 2, block_index: 0 },
        Instruction::BlockStart { level_index: 1, block_index: 1, items_count: 3, },
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 0 }, // 14
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 1 }, // 15
        Instruction::WriteItem { level_index: 1, block_index: 1, item_index: 2 }, // 16
        Instruction::BlockFinish { level_index: 1, block_index: 1 },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 1 }, // 17
        Instruction::BlockStart { level_index: 1, block_index: 2, items_count: 3, },
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 0 }, // 18
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 1 }, // 19
        Instruction::WriteItem { level_index: 1, block_index: 2, item_index: 2 }, // 20
        Instruction::BlockFinish { level_index: 1, block_index: 2 },
        Instruction::WriteItem { level_index: 0, block_index: 0, item_index: 2 }, // 21
        Instruction::BlockFinish { level_index: 0, block_index: 0 },
        Instruction::Done,
    ]);
}

#[derive(PartialEq, Debug)]
enum Instruction {
    TreeStart,
    BlockStart { level_index: usize, block_index: usize, items_count: usize, },
    WriteItem { level_index: usize, block_index: usize, item_index: usize, },
    BlockFinish { level_index: usize, block_index: usize, },
    Done,
}

fn interpret_script(sketch: &sketch::Tree, mut script: Vec<Instruction>) {
    script.reverse();

    let mut plan_ctx = plan::Context::new(sketch);
    let mut kont = plan::Script::boot();

    assert_eq!(script.pop(), Some(Instruction::TreeStart));
    loop {
        use plan::{Perform, Op};
        match kont.next.step(&mut plan_ctx) {
            plan::Instruction::Perform(Perform { op: Op::BlockStart { items_count, }, level_index, block_index, next, }) => {
                assert_eq!(script.pop(), Some(Instruction::BlockStart { level_index, block_index, items_count, }));
                kont = next;
            },
            plan::Instruction::Perform(
                Perform { op: Op::BlockItem { index: item_index, }, level_index, block_index, next, },
            ) => {
                assert_eq!(script.pop(), Some(Instruction::WriteItem { level_index, block_index, item_index, }));
                kont = next;
            },
            plan::Instruction::Perform(Perform { op: Op::BlockFinish, level_index, block_index, next, }) => {
                assert_eq!(script.pop(), Some(Instruction::BlockFinish { level_index, block_index, }));
                kont = next;
            },
            plan::Instruction::Done => {
                assert_eq!(script.pop(), Some(Instruction::Done));
                break
            },
        }
    }
}
