use super::super::{
    fold,
    plan,
    super::sketch,
};

#[test]
fn tree17_4() {
    let sketch = sketch::Tree::new(17, 4);
    assert_eq!(interpret_fold(&sketch), vec![

    ]);
}

fn interpret_fold(sketch: &sketch::Tree) -> Vec<(usize, usize)> {
    let mut plan_op = plan::Script::start()
        .step(sketch);
    let mut fold_op = fold::Script::start()
        .step(plan_op, sketch).unwrap();
    loop {
        match fold_op {
            fold::Instruction::Perform(..) =>
                unimplemented!(),
            fold::Instruction::Done(done) =>
                return done.levels_iter().collect(),
        }
    }
}
