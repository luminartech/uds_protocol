/// A trait for types that can be serialized and deserialized from a byte stream.
/// Note that the `ITERABLE` constant is used to declare whether the type is iterable.
///
/// If `ITERABLE` is `true`, then the type is expected to be deserialized as a sequence of values.
/// If `ITERABLE` is `false`, then the type is expected to be deserialized as a single value.
///
pub trait WireFormat<E>: Sized
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
    T: WireFormat<E>,
    E: std::error::Error,
{
    reader: &'a mut R,
    _phantom: std::marker::PhantomData<T>,
    _phantom2: std::marker::PhantomData<E>,
}

impl<T: WireFormat<E>, E: std::error::Error, R: std::io::Read> Iterator
    for WireFormatIterator<'_, T, E, R>
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

pub trait IterableWireFormat<E>: WireFormat<E>
where
    E: std::error::Error,
{
    fn from_reader_iterable<T: std::io::Read>(
        reader: &mut T,
    ) -> impl Iterator<Item = Result<Self, E>> {
        WireFormatIterator {
            reader,
            _phantom: std::marker::PhantomData,
            _phantom2: std::marker::PhantomData,
        }
    }
}

pub trait SingleValueWireFormat<E>: WireFormat<E>
where
    E: std::error::Error,
{
    fn from_reader<T: std::io::Read>(reader: &mut T) -> Result<Self, E> {
        Ok(Self::option_from_reader(reader)?.expect(
            "SingleValueWireFormat is only valid to implement on types which never return none",
        ))
    }
}
