use std::borrow::Borrow;
use std::iter::Iterator;
use std::collections::BinaryHeap;
use std::cmp::{Ordering, PartialEq, PartialOrd, Ord};

#[derive(Debug)]
pub enum BuildError<IE, TE> {
    Iter(IE),
    NTree(TE),
}

pub trait NTreeWriter: Sized {
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

    fn build<I, E>(mut self, mut src: I, src_len: usize, min_tree_height: usize, max_block_size: usize) ->
        Result<Self::Result, BuildError<E, Self::Error>> where I: Iterator<Item = Result<Self::Item, E>>
    {
        let mut index_block_size = max_block_size + 1;
        let mut tree_height = min_tree_height;
        while src_len > 0 && index_block_size > max_block_size {
            index_block_size = (src_len as f64).powf(1.0 / tree_height as f64) as usize;
            tree_height += 1;
        }
        let root_block_pos = build_block(&mut src, 0, src_len, index_block_size, &mut self)?;
        self.finish(root_block_pos).map_err(|e| BuildError::NTree(e))
    }
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

struct Pos<C, Q> {
    cursor: C,
    key: Q,
}

impl<C, Q> Eq for Pos<C, Q> where C: PartialEq { }

impl<C, Q> PartialEq for Pos<C, Q> where C: PartialEq {
    fn eq(&self, other: &Pos<C, Q>) -> bool {
        self.cursor.eq(&other.cursor)
    }
}

impl<C, Q> PartialOrd for Pos<C, Q> where C: PartialOrd {
    fn partial_cmp(&self, other: &Pos<C, Q>) -> Option<Ordering> {
        self.cursor.partial_cmp(&other.cursor)
    }
}

impl<C, Q> Ord for Pos<C, Q> where C: PartialOrd {
    fn cmp(&self, other: &Pos<C, Q>) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

pub trait NTreeReader {
    type Item;
    type Error;
    type Cursor: PartialOrd;

    fn root(&self) -> Self::Cursor;
    fn load<'a>(&'a mut self, cursor: &mut Self::Cursor) -> Result<Option<&'a Self::Item>, Self::Error>;
    fn advance(&self, cursor: Self::Cursor) -> Self::Cursor;
    fn jump(&self, cursor: Self::Cursor) -> Self::Cursor;

    fn lookup_iter<'r, 'q: 'r, I, Q: ?Sized + 'q>(&'r mut self, keys_iter: I) -> NTreeIter<Self, Q> where I: Iterator<Item = &'q Q> {
        NTreeIter {
            queue: keys_iter.map(|key| Pos { cursor: self.root(), key: key, }).collect(),
            tree: self,
        }
    }
}

pub struct NTreeIter<'t, 'q: 't, T: ?Sized + 't, Q: ?Sized + 'q> where T: NTreeReader {
    tree: &'t mut T,
    queue: BinaryHeap<Pos<T::Cursor, &'q Q>>,
}

impl<'t, 'q: 't, T: ?Sized + 't, Q: ?Sized + 'q> NTreeIter<'t, 'q, T, Q> where T: NTreeReader {
    pub fn next<'i>(&'i mut self) -> Result<Option<(&'q Q, &'i T::Item)>, T::Error>
        where T::Item: Borrow<Q>, Q: PartialOrd
    {
        loop {
            if let Some(mut pos) = self.queue.pop() {
                enum Then { Result, Jump, Advance, Skip, };
                let action = match self.tree.load(&mut pos.cursor)? {
                    Some(block_item) if pos.key == block_item.borrow() =>
                        Then::Result,
                    Some(block_item) if pos.key < block_item.borrow() =>
                        Then::Jump,
                    Some(..) =>
                        Then::Advance,
                    None =>
                        Then::Skip,
                };

                match action {
                    Then::Result =>
                        return Ok(self.tree.load(&mut pos.cursor)?.map(|block_item| (pos.key, block_item))),
                    Then::Jump =>
                        self.queue.push(Pos { cursor: self.tree.jump(pos.cursor), key: pos.key, }),
                    Then::Advance =>
                        self.queue.push(Pos { cursor: self.tree.advance(pos.cursor), key: pos.key, }),
                    Then::Skip =>
                        (),
                }
            } else {
                return Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
