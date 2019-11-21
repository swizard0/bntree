use std::borrow::Borrow;
use std::collections::BinaryHeap;
use std::cmp::{Ordering, PartialEq, PartialOrd, Ord};

pub trait Reader {
    type Item;
    type Error;
    type Cursor: PartialOrd;

    fn root(&self) -> Result<Option<Self::Cursor>, Self::Error>;
    fn access<'a>(&'a self, cursor: &Self::Cursor) -> &'a Self::Item;
    fn advance(&mut self, cursor: Self::Cursor) -> Result<Option<Self::Cursor>, Self::Error>;
    fn jump(&mut self, cursor: Self::Cursor) -> Result<Option<Self::Cursor>, Self::Error>;
}

pub struct LookupBatch<'r, 'q, C, R, Q>
{
    tree_reader: &'r mut R,
    queue: BinaryHeap<Pos<C, &'q Q>>,
}

impl<R, Q> LookupBatch<'_, '_, R::Cursor, R, Q> where R: Reader {
    pub fn new<'r, 'q, I>(tree_reader: &'r mut R, keys_iter: I) ->
        Result<LookupBatch<'r, 'q, R::Cursor, R, Q>, R::Error>
    where I: Iterator<Item = &'q Q>
    {
        let mut queue = BinaryHeap::new();
        for key in keys_iter {
            if let Some(cursor) = tree_reader.root()? {
                queue.push(Pos { cursor, key, });
            }
        }
        Ok(LookupBatch { queue, tree_reader, })
    }
}

impl<'r, 'q, R, Q> LookupBatch<'r, 'q, R::Cursor, R, Q> where R: Reader,
{
    pub fn next<'s>(&'s mut self) -> Result<Option<(&'q Q, &'s R::Item)>, R::Error>
    where R::Item: Borrow<Q>,
          Q: PartialOrd
    {
        loop {
            if let Some(pos) = self.queue.pop() {
                enum Action { Access, Advance, Jump, }
                let action = {
                    let block_item = self.tree_reader.access(&pos.cursor);
                    if pos.key == block_item.borrow() {
                        Action::Access
                    } else if pos.key < block_item.borrow() {
                        Action::Advance
                    } else {
                        Action::Jump
                    }
                };
                let cursor_next = match action {
                    Action::Access => {
                        let block_item = self.tree_reader.access(&pos.cursor);
                        return Ok(Some((pos.key, block_item)));
                    },
                    Action::Advance =>
                        self.tree_reader.jump(pos.cursor)?,
                    Action::Jump =>
                        self.tree_reader.advance(pos.cursor)?,
                };
                if let Some(cursor) = cursor_next {
                    self.queue.push(Pos { cursor, ..pos });
                }
            } else {
                return Ok(None)
            }
        }
    }
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

pub struct InOrderTraverse<R> {
    tree_reader: R,
}

impl<R> InOrderTraverse<R> {
    pub fn new(tree_reader: R) -> InOrderTraverse<R> {
        InOrderTraverse {
            tree_reader,
        }
    }

    pub fn iter(self) -> InOrderIterator<R, R::Cursor, R::Error> where R: Reader {
        let script = vec![
            TraverseAction::Inspect(
                self.tree_reader.root(),
            ),
        ];

        InOrderIterator {
            tree_reader: self.tree_reader,
            script,
        }
    }
}

enum TraverseAction<C, E> {
    Inspect(Result<Option<C>, E>),
    Return(C),
}


pub struct InOrderIterator<R, C, E> {
    tree_reader: R,
    script: Vec<TraverseAction<C, E>>,
}

impl<R> Iterator for InOrderIterator<R, R::Cursor, R::Error>
where R: Reader,
      R::Item: Clone,
      R::Cursor: Clone,
{
    type Item = Result<R::Item, R::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(command) = self.script.pop() {
            match command {
                TraverseAction::Inspect(Err(error)) =>
                    return Some(Err(error)),
                TraverseAction::Inspect(Ok(None)) =>
                    (),
                TraverseAction::Inspect(Ok(Some(cursor))) => {
                    self.script.push(TraverseAction::Return(
                        cursor.clone(),
                    ));
                    self.script.push(TraverseAction::Inspect(
                        self.tree_reader.jump(cursor),
                    ));
                },
                TraverseAction::Return(cursor) => {
                    let item = self.tree_reader.access(&cursor).clone();
                    self.script.push(TraverseAction::Inspect(
                        self.tree_reader.advance(cursor),
                    ));
                    return Some(Ok(item));
                },
            };
        }
        None
    }
}

pub struct BlocksBfsTraverse<R> {
    tree_reader: R,
}

impl<R> BlocksBfsTraverse<R> {
    pub fn new(tree_reader: R) -> BlocksBfsTraverse<R> {
        BlocksBfsTraverse {
            tree_reader,
        }
    }

    pub fn iter(self) -> BlocksBfsIterator<R, R::Cursor, R::Error> where R: Reader {
        BlocksBfsIterator {
            tree_reader: self.tree_reader,
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct BlocksBfsIterator<R, C, E> {
    tree_reader: R,
    _marker: std::marker::PhantomData<(C, E)>,
}
