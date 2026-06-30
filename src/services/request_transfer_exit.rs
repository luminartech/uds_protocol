//! `RequestTransferExit` (0x37 / 0x77) service implementation.
use crate::{Decode, Encode, Error};

macro_rules! transfer_exit_descriptor {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[non_exhaustive]
        pub struct $name<'d> {
            /// The optional, opaque parameter record (empty slice if absent).
            pub parameter_record: &'d [u8],
        }
        impl<'d> $name<'d> {
            /// Create from the optional parameter record (empty slice if absent).
            #[must_use]
            pub const fn new(parameter_record: &'d [u8]) -> Self {
                Self { parameter_record }
            }
        }
        impl Encode for $name<'_> {
            fn encoded_size(&self) -> usize {
                self.parameter_record.len()
            }
            fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
                writer.write_all(self.parameter_record).map_err(Error::io)?;
                Ok(self.parameter_record.len())
            }
        }
        impl<'a> Decode<'a> for $name<'a> {
            fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
                Ok((
                    Self {
                        parameter_record: buf,
                    },
                    &[],
                ))
            }
        }
    };
}

transfer_exit_descriptor!(
    RequestTransferExitRequest,
    "Request to exit a transfer, carrying an optional parameter record."
);
transfer_exit_descriptor!(
    RequestTransferExitResponse,
    "Positive response to `RequestTransferExit`, carrying an optional parameter record."
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};

    #[test]
    fn rte_request_round_trips_with_and_without_record() {
        for rec in [&[][..], &[0xAA, 0xBB][..]] {
            let req = RequestTransferExitRequest::new(rec);
            let mut buf = [0u8; 8];
            let n = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
            assert_eq!(&buf[..n], rec);
            let (d, rest) = <RequestTransferExitRequest as Decode>::decode(&buf[..n]).unwrap();
            assert!(rest.is_empty());
            assert_eq!(d.parameter_record, rec);
            assert_encode_size_agrees(&req);
        }
    }
}
