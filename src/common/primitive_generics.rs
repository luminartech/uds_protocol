use crate::{Error, SingleValueWireFormat, WireFormat};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use clap::Parser;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[macro_export]
macro_rules! generate_uds_primitive_1_byte {
    ( $type_name:ident, $number_type:ty, $read_fn:ident, $write_fn:ident) => {
        #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, Parser, ToSchema)]
        pub struct $type_name {
            pub value: $number_type,
        }

        impl $type_name {
            pub fn new(value: $number_type) -> Self {
                Self { value }
            }
        }

        impl WireFormat for $type_name {
            fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
                let value = reader.$read_fn()?;
                Ok(Some(Self::new(value)))
            }

            fn required_size(&self) -> usize {
                1
            }

            fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
                writer.$write_fn(self.value)?;
                Ok(self.required_size())
            }
        }

        impl SingleValueWireFormat for $type_name {}
    };
}

#[macro_export]
macro_rules! generate_uds_primitive_greater_than_1_byte {
    ( $type_name:ident, $number_type:ty, $read_fn:ident, $write_fn:ident , $endian_type:ty) => {
        #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, Parser, ToSchema)]
        pub struct $type_name {
            pub value: $number_type,
        }

        impl $type_name {
            pub fn new(value: $number_type) -> Self {
                Self { value }
            }
        }

        impl WireFormat for $type_name {
            fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
                let value = reader.$read_fn::<$endian_type>()?;
                Ok(Some(Self::new(value)))
            }

            fn required_size(&self) -> usize {
                std::mem::size_of::<$number_type>()
            }

            fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
                writer.$write_fn::<$endian_type>(self.value)?;
                Ok(self.required_size())
            }
        }

        impl SingleValueWireFormat for $type_name {}
    };
}

generate_uds_primitive_1_byte!(U8, u8, read_u8, write_u8);
generate_uds_primitive_1_byte!(I8, i8, read_i8, write_i8);
generate_uds_primitive_greater_than_1_byte!(U16, u16, read_u16, write_u16, BigEndian);
generate_uds_primitive_greater_than_1_byte!(I16, i16, read_i16, write_i16, BigEndian);
generate_uds_primitive_greater_than_1_byte!(U32, u32, read_u32, write_u32, BigEndian);
generate_uds_primitive_greater_than_1_byte!(I32, i32, read_i32, write_i32, BigEndian);
generate_uds_primitive_greater_than_1_byte!(F32, f32, read_f32, write_f32, BigEndian);
generate_uds_primitive_greater_than_1_byte!(U64, u64, read_u64, write_u64, BigEndian);
generate_uds_primitive_greater_than_1_byte!(I64, i64, read_i64, write_i64, BigEndian);
generate_uds_primitive_greater_than_1_byte!(F64, f64, read_f64, write_f64, BigEndian);
