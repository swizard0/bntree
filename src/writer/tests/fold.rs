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
    let mut fold_ctx = Default::default();
    let plan_op = plan::Script::new()
        .step(sketch);
    let mut fold_op = fold::Script::new()
        .step(&mut fold_ctx, fold::StepArg { op: plan_op, sketch, }).unwrap();
    loop {
        match fold_op {
            fold::Instruction::Perform(fold::Perform { op: fold::Op::VisitLevel(fold::VisitLevel { next, .. }), next_plan, }) =>
                fold_op = next.level_ready(0, &mut fold_ctx, fold::StepArg { op: next_plan, sketch, }).unwrap(),
            fold::Instruction::Perform(fold::Perform { op: fold::Op::VisitBlockStart(fold::VisitBlockStart { level_seed, next, .. }), next_plan, }) =>
                fold_op = next.block_ready(level_seed, &mut fold_ctx, fold::StepArg { op: next_plan, sketch, }).unwrap(),
            fold::Instruction::Perform(fold::Perform { op: fold::Op::VisitItem(fold::VisitItem { level_seed, next, .. }), next_plan, }) =>
                fold_op = next.item_ready(level_seed + 1, &mut fold_ctx, fold::StepArg { op: next_plan, sketch, }).unwrap(),
            fold::Instruction::Perform(fold::Perform { op: fold::Op::VisitBlockFinish(fold::VisitBlockFinish { level_seed, next, .. }), next_plan, }) =>
                fold_op = next.block_flushed(level_seed, &mut fold_ctx, fold::StepArg { op: next_plan, sketch, }).unwrap(),
            fold::Instruction::Done =>
                return fold_ctx.levels_iter().collect(),
        }
    }
}
