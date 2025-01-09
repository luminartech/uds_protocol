/// A trait for types that can be serialized and deserialized from a byte stream.
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
    fn from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, E>;

    /// Serialize a value to a byte stream.
    /// Returns the number of bytes written.
    /// # Errors
    /// - If the data cannot be written to the stream
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, E>;
}
