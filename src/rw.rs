use super::{
    reader,
    writer,
};

#[derive(Debug)]
pub enum ReverseError<RE, WE> {
    Reader(RE),
    Writer(WE),
}

pub fn reverse<R, W>(
    reader: R,
    _writer: W,
)
    -> Result<W::Result, ReverseError<R::Error, W::Error>>
where R: reader::Reader,
      W: writer::Writer<Item = R::Item>,
      R::Item: Clone,
      R::Cursor: Clone,
{
    for maybe_item in reader::InOrderTraverse::new(reader).iter() {
        let _item = maybe_item.map_err(ReverseError::Reader)?;

    }

    unimplemented!()
}
