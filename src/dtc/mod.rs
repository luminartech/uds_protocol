//! DTC (Diagnostic Trouble Code) vocabulary shared across the DTC services
//! (`ReadDTCInformation`, `ClearDiagnosticInformation`).
mod status;
pub use status::*;

mod snapshot;
pub use snapshot::*;

mod ext_data;
pub use ext_data::*;
