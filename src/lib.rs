pub mod reader;

#[derive(Debug)]
pub enum BuildError<IE, TE> {
    Iter(IE),
    NTree(TE),
}

pub trait NTreeWriter {
    type Item;
    type Error;
    type Position;
    type Block;
    type Result;

    fn empty_pos(&self) -> Self::Position;
    fn make_block(&self) -> Result<Self::Block, Self::Error>;
    fn write_item(&mut self, block: &mut Self::Block, item: Self::Item, child_block_pos: Self::Position) -> Result<(), Self::Error>;
    fn flush_block(&mut self, block: Self::Block) -> Result<Self::Position, Self::Error>;
    fn finish(self, root_block_pos: Self::Position) -> Result<Self::Result, Self::Error>;
}

pub fn build<I, W, E>(
    mut src: I,
    src_len: usize,
    mut dst: W,
    min_tree_height: usize,
    max_block_size: usize,
) ->
    Result<W::Result, BuildError<E, W::Error>>
where
    I: Iterator<Item = Result<W::Item, E>>,
    W: NTreeWriter
{
    let mut index_block_size = max_block_size + 1;
    let mut tree_height = min_tree_height;
    while src_len > 0 && index_block_size > max_block_size {
        index_block_size = (src_len as f64).powf(1.0 / tree_height as f64) as usize;
        tree_height += 1;
    }
    let root_block_pos = build_block(&mut src, 0, src_len, index_block_size, &mut dst)?;
    dst.finish(root_block_pos).map_err(|e| BuildError::NTree(e))
}

fn build_block<T, E, P, I, IE, W>(src: &mut I, block_start: usize, block_end: usize, block_size: usize, dst: &mut W) -> Result<P, BuildError<IE, E>>
    where I: Iterator<Item = Result<T, IE>>, W: NTreeWriter<Item = T, Error = E, Position = P>
{
    let interval = block_end - block_start;
    let (index_start, index_inc) = if interval > block_size {
        (interval - (interval / block_size * (block_size - 1)) - 1, interval / block_size)
    } else {
        (0, 1)
    };

    let mut block = dst.make_block().map_err(|e| BuildError::NTree(e))?;
    let mut node_block_start = block_start;
    let mut node_block_end = block_start + index_start;
    while node_block_end < block_end {
        let child_block_pos = if node_block_start < node_block_end {
            build_block(src, node_block_start, node_block_end, block_size, dst)?
        } else {
            dst.empty_pos()
        };

        if let Some(maybe_item) = src.next() {
            dst.write_item(&mut block, maybe_item.map_err(|e| BuildError::Iter(e))?, child_block_pos).map_err(|e| BuildError::NTree(e))?;
        } else {
            break
        }

        node_block_start = node_block_end + 1;
        node_block_end += index_inc;
    }

    dst.flush_block(block).map_err(|e| BuildError::NTree(e))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
