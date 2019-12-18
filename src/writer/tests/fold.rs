use super::super::{
    fold,
    plan,
    super::sketch,
};

#[test]
fn tree17_4() {
    let sketch = sketch::Tree::new(17, 4);
    assert_eq!(interpret_fold_count_items(&sketch), vec![(0, 4), (1, 13)]);
}

#[test]
fn tree17_3() {
    let sketch = sketch::Tree::new(17, 3);
    assert_eq!(interpret_fold_count_items(&sketch), vec![(0, 3), (1, 9), (2, 5)]);
}

fn interpret_fold_count_items(sketch: &sketch::Tree) -> Vec<(usize, usize)> {
    let mut fold_ctx = fold::Context::new(
        plan::Context::new(sketch),
        sketch,
    );

    let mut kont = fold::Script::boot();
    loop {
        let plan_op = match kont.plan_action {
            fold::PlanAction::Idle(plan_op) =>
                plan_op,
            fold::PlanAction::Step(plan::Continue { next: plan_script, }) =>
                plan_script.step(fold_ctx.plan_ctx()),
        };
        kont = match kont.next.step(&mut fold_ctx, plan_op).unwrap() {
            fold::Instruction::Op(fold::Op::VisitLevel(fold::VisitLevel { next, .. })) =>
                next.level_ready(0, &mut fold_ctx).unwrap(),
            fold::Instruction::Op(fold::Op::VisitBlockStart(fold::VisitBlockStart { level_seed, next, .. })) =>
                next.block_ready(level_seed, &mut fold_ctx).unwrap(),
            fold::Instruction::Op(fold::Op::VisitItem(fold::VisitItem { level_seed, next, .. })) =>
                next.item_ready(level_seed + 1, &mut fold_ctx).unwrap(),
            fold::Instruction::Op(fold::Op::VisitBlockFinish(fold::VisitBlockFinish { level_seed, next, .. })) =>
                next.block_flushed(level_seed, &mut fold_ctx).unwrap(),
            fold::Instruction::Done => {
                let (_plan_ctx, fold_iter) =
                    fold_ctx.into_levels_iter();
                return fold_iter.collect();
            },
        };
    }
}
