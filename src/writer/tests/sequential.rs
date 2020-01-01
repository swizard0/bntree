use std::io::Write;

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
                    sequential::Pass::Markup(sequential::ActionMarkup {
                        context: markup_ctx,
                        next: two_pass::markup::Continue {
                            fold_action: two_pass::markup::FoldAction::Idle(fold_op),
                            next: markup_next,
                        },
                    }) =>
                        markup_next.step(markup_ctx, fold_op).unwrap().into(),
                    sequential::Pass::Markup(sequential::ActionMarkup {
                        context: markup_ctx,
                        next: two_pass::markup::Continue {
                            fold_action: two_pass::markup::FoldAction::Step(fold::Continue {
                                plan_action: fold::PlanAction::Idle(plan_op),
                                next: fold_next,
                            }),
                            next: markup_next,
                        },
                    }) => {
                        let fold_op = fold_next.step(markup_ctx.fold_ctx(), plan_op).unwrap();
                        markup_next.step(markup_ctx, fold_op).unwrap().into()
                    },
                    sequential::Pass::Markup(sequential::ActionMarkup {
                        context: markup_ctx,
                        next: two_pass::markup::Continue {
                            fold_action: two_pass::markup::FoldAction::Step(fold::Continue {
                                plan_action: fold::PlanAction::Step(plan::Continue { next: plan_next, }),
                                next: fold_next,
                            }),
                            next: markup_next,
                        },
                    }) => {
                        let plan_op = plan_next.step(markup_ctx.fold_ctx().plan_ctx());
                        let fold_op = fold_next.step(markup_ctx.fold_ctx(), plan_op).unwrap();
                        markup_next.step(markup_ctx, fold_op).unwrap().into()
                    },
                    sequential::Pass::Write(sequential::ActionWrite { context, next, }) =>
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
            sequential::Instruction::Op(sequential::Op::WriteTreeHeader(write_tree_header)) => {
                write_tree_header
                    .block_writer
                    .write_fmt(format_args!("tree header"))
                    .unwrap();
                write_tree_header.next.tree_header_written(&mut context).unwrap()
            },
            sequential::Instruction::Op(sequential::Op::FinishTreeHeader(finish_tree_header)) =>
                finish_tree_header.next.tree_header_finished(&mut context).unwrap(),
            sequential::Instruction::Done =>
                break,
        }
    }
}
