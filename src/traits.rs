/// A trait for types that can be serialized and deserialized from a byte stream.
pub trait WireFormat<E>: Sized
where
    E: std::error::Error,
{
    fn from_reader<T: std::io::Read>(reader: &mut T) -> Result<Self, E>;
    fn from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, E>;

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, E>;
}
