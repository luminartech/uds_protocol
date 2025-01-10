struct WireFormatIterator<'a, T, E, R: std::io::Read>
where
    T: WireFormat<E>,
    E: std::error::Error,
{
    reader: &'a mut R,
    _phantom: std::marker::PhantomData<T>,
    _phantom2: std::marker::PhantomData<E>,
}

impl<'a, T: WireFormat<E>, E: std::error::Error, R: std::io::Read> Iterator
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

/// A trait for types that can be serialized and deserialized from a byte stream.
pub trait WireFormat<E>: Sized
where
    E: std::error::Error,
{
    const ITERABLE: bool;
    /// Deserialize a value from a byte stream.
    /// Returns Ok(`Some(value)`) if the stream contains a complete value.
    /// Returns Ok(`None`) if the stream is empty.
    /// # Errors
    /// - if the stream is not in the expected format
    /// - if the stream contains partial data
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, E>;

    /// Cases:
    /// - O or more:
    /// - 1 or more
    /// - Exactly 1

    fn from_reader_iterable<T: std::io::Read>(
        reader: &mut T,
    ) -> impl Iterator<Item = Result<Self, E>> {
        assert!(
            Self::ITERABLE,
            "from_reader_iterable is only callable on iterable WireFormat types"
        );
        WireFormatIterator {
            reader,
            _phantom: std::marker::PhantomData,
            _phantom2: std::marker::PhantomData,
        }
    }

    fn from_reader<T: std::io::Read>(reader: &mut T) -> Result<Self, E> {
        assert!(
            !Self::ITERABLE,
            "Cannot call from_reader on an iterable type"
        );
        Ok(Self::option_from_reader(reader)?.unwrap())
    }

    /// Serialize a value to a byte stream.
    /// Returns the number of bytes written.
    /// # Errors
    /// - If the data cannot be written to the stream
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, E>;
}
