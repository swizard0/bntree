use super::{
    sketch,
    plan,
    fold,
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
