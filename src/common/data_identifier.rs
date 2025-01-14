use crate::{Error, SingleValueWireFormat, WireFormat};
use std::fmt::Debug;

enum DataIdentifier<U>
where
    U: SingleValueWireFormat<Error> + Debug,
{
    // TODO: ISO Spec C.1 DID parameter definitions
    UserDefined(U),
}

impl<U: SingleValueWireFormat<Error> + Debug> WireFormat<Error> for DataIdentifier<U> {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        // let DataIdentifier<U>
        // U::option_from_reader(reader);
        Ok(None)
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        Ok(0)
    }
}
