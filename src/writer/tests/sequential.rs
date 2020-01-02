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
        kont = match kont.step_rec(&mut context).unwrap() {
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
