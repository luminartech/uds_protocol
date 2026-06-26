// Fuzz the Response decode path with arbitrary bytes.
// Any panic here indicates a bug — decode should return Err, not crash.
#![no_main]
use libfuzzer_sys::fuzz_target;
use uds_protocol::{Decode, Response};

fuzz_target!(|data: &[u8]| {
    // Attempt to decode arbitrary bytes as a UDS response.
    // We don't care about the result — only that it doesn't panic.
    let _ = Response::decode(data);
});
