// Fuzz the Request decode path with arbitrary bytes.
// Any panic here indicates a bug — decode should return Err, not crash.
#![no_main]
use libfuzzer_sys::fuzz_target;
use uds_protocol::{Decode, Request};

fuzz_target!(|data: &[u8]| {
    // Attempt to decode arbitrary bytes as a UDS request.
    // We don't care about the result — only that it doesn't panic.
    // Reporting SPRMIB indexes into the payload of an unmodeled service, so run it on every
    // frame that decodes. It is panic-free by construction today -- `data.first()`, not
    // `data[0]` -- which is exactly why it is worth pinning before an edit changes that.
    if let Ok((request, _)) = Request::decode(data) {
        let _ = request.is_positive_response_suppressed();
    }
});
