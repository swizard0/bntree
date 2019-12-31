use super::super::{
    plan,
    fold,
    two_pass,
    super::sketch,
    traverse::sequential,
};

#[test]
fn tree17_4_markup() {
    let sketch = sketch::Tree::new(17, 4);

    let mut context = sequential::Context::new();
    let mut kont = sequential::Script::boot();
    loop {
        let underlying_op = match kont.underlying_action {
            sequential::UnderlyingAction::Idle(underlying_op) =>
                underlying_op,
            sequential::UnderlyingAction::Step(underlying_step) =>
                match underlying_step.action(&mut context).unwrap() {
                    sequential::Pass::Markup(sequential::ActionMarkup { context, op, }) =>
                        unimplemented!(),
                    sequential::Pass::Write(sequential::ActionWrite { context, op, }) =>
                        unimplemented!(),
                },
        };
        kont = match kont.next.step(&mut context, underlying_op).unwrap() {
            sequential::Instruction::Op(sequential::Op::MarkupStart(markup_start)) =>
                markup_start.created_context(
                    two_pass::markup::Context::new(
                        fold::Context::new(plan::Context::new(&sketch), &sketch),
                    ),
                    &mut context,
                ),
            sequential::Instruction::Op(sequential::Op::WriteTreeHeader(write_tree_header)) =>
                unimplemented!(),
            sequential::Instruction::Done =>
                break,
        }
    }
}
