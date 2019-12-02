use bincode;

use super::super::super::sketch;
use super::in_memory;

#[test]
fn tree17_4_in_memory() {
    let sketch = sketch::Tree::new(17, 4);
    let mut instruction = in_memory::build(&sketch);
    let mut source = 0 .. 17;
    loop {
        match instruction {
            in_memory::Instruction::PassMarkup(fsm) =>
                instruction = fsm.next(),
            in_memory::Instruction::PassWrite(fsm) => {
                source = 0 .. 17;
                instruction = fsm.next();
            },
            in_memory::Instruction::WriteItem(in_memory::WriteItem { mut block_writer, next, }) => {
                let item = source.next().unwrap();
                bincode::serialize_into(&mut block_writer, &item).unwrap();
                instruction = next.item_written(block_writer).next();
            },
            in_memory::Instruction::FlushBlock(next) =>
                instruction = next.block_flushed().next(),
            in_memory::Instruction::Done(memory) => {
                assert_eq!(memory, vec![
                    13, 104, 122, 139, 122, 154, 15, 104, 2, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
                    4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
                    4, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0, 0, 0, 0, 0,
                    4, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 0, 0, 0, 0,
                    4, 0, 0, 0, 4, 0, 0, 0, 1, 104, 0, 0, 0, 0, 0, 0,
                    0, 9, 0, 0, 0, 1, 128, 0, 0, 0, 0, 0, 0, 0, 14, 0,
                    0, 0, 1, 152, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 1,
                    176, 0, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 7, 0,
                    0, 0, 0, 8, 0, 0, 0, 0, 4, 0, 0, 0, 10, 0, 0, 0,
                    0, 11, 0, 0, 0, 0, 12, 0, 0, 0, 0, 13, 0, 0, 0, 0,
                    1, 0, 0, 0, 15, 0, 0, 0, 0
                ]);
                break;
            },
        }
    }
}
