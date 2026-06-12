// Fuzz encode→decode roundtrip: encode a successfully-decoded request,
// then decode it again and verify the result matches.
#![no_main]
use libfuzzer_sys::fuzz_target;
use uds_protocol::{ProtocolRequest, SingleValueWireFormat, WireFormat};

fuzz_target!(|data: &[u8]| {
    // Only proceed if we can decode the input
    let Ok(request) = ProtocolRequest::decode(&mut &data[..]) else {
        return;
    };

    // Encode it back
    let mut buf = Vec::with_capacity(request.required_size() + 1);
    if request.encode(&mut buf).is_err() {
        return;
    }

    // Decode the re-encoded form
    let Ok(roundtripped) = ProtocolRequest::decode(&mut buf.as_slice()) else {
        panic!("Failed to decode a message that was just encoded successfully");
    };

    // The roundtripped value must equal the original
    assert_eq!(
        request, roundtripped,
        "Roundtrip mismatch: encode(decode(data)) != decode(data)"
    );
});
