/// A trait for types that can be deserialized from a
/// [`Reader`](https://doc.rust-lang.org/std/io/trait.Read.html) and serialized
/// to a [`Writer`](https://doc.rust-lang.org/std/io/trait.Write.html).
///
/// `WireFormat` acts as the base trait for all types that can be serialized and deserialized
/// as part of the UDS Protocol ecosystem.
///
/// Some types need the ability to be deserialized without knowing the size of the data in advance.
/// To support this, the `option_from_reader` function returns an `Option<Self>`.
/// If the reader contains a complete value, it returns `Some(value)`.
/// If the reader is completely empty, it returns `None`.
/// Many types will never return `None`, and for these types, the `SingleValueWireFormat`,
/// trait can be implemented, providing a more ergonomic API.
pub trait WireFormat<'a, E>:
    Sized + std::fmt::Debug + serde::Deserialize<'a> + serde::Serialize
where
    E: std::error::Error,
{
    /// Deserialize a value from a byte stream.
    /// Returns Ok(`Some(value)`) if the stream contains a complete value.
    /// Returns Ok(`None`) if the stream is empty.
    /// # Errors
    /// - if the stream is not in the expected format
    /// - if the stream contains partial data
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, E>;

    /// Serialize a value to a byte stream.
    /// Returns the number of bytes written.
    /// # Errors
    /// - If the data cannot be written to the stream
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, E>;
}

struct WireFormatIterator<'a, T, E, R: std::io::Read>
where
    T: WireFormat<'a, E>,
    E: std::error::Error,
{
    reader: &'a mut R,
    _phantom: std::marker::PhantomData<T>,
    _phantom2: std::marker::PhantomData<E>,
}

/// For types that can appear in lists of unknown length, this trait provides an iterator
/// that can be used to deserialize a stream of values.
impl<'a, T: WireFormat<'a, E>, E: std::error::Error, R: std::io::Read> Iterator
    for WireFormatIterator<'a, T, E, R>
where
    R: std::io::Read,
{
    type Item = Result<T, E>;
    fn next(&mut self) -> Option<Self::Item> {
        match T::option_from_reader(self.reader.by_ref()) {
            Ok(Some(value)) => Some(Ok(value)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

pub trait IterableWireFormat<'a, E>: WireFormat<'a, E>
where
    E: std::error::Error,
{
    fn from_reader_iterable<T: std::io::Read>(
        reader: &'a mut T,
    ) -> impl Iterator<Item = Result<Self, E>> {
        WireFormatIterator {
            reader,
            _phantom: std::marker::PhantomData,
            _phantom2: std::marker::PhantomData,
        }
    }
}

/// A trait for types that will always deserialize to a single value or error.
/// Types which might return `None` when deserialized should not implement this trait.
pub trait SingleValueWireFormat<'a, E>: WireFormat<'a, E>
where
    E: std::error::Error,
{
    fn from_reader<T: std::io::Read>(reader: &mut T) -> Result<Self, E> {
        Ok(Self::option_from_reader(reader)?.expect(
            "SingleValueWireFormat is only valid to implement on types which never return none",
        ))
    }
}
