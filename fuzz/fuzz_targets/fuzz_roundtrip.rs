// Fuzz encode→decode roundtrip: decode arbitrary bytes into a Request,
// re-encode it, and verify the encoding is idempotent at the byte level.
//
// `Request` does not implement `PartialEq`, so we compare the canonical wire
// bytes rather than the decoded values: encoding a decoded message, decoding
// those bytes again, and re-encoding must yield identical bytes.
#![no_main]
use libfuzzer_sys::fuzz_target;
use uds_protocol::{Decode, Encode, Request};

fuzz_target!(|data: &[u8]| {
    // Only proceed if we can decode the input.
    let Ok((request, _rest)) = Request::decode(data) else {
        return;
    };

    // Encode the decoded request into its canonical wire form.
    let mut first = vec![0u8; request.encoded_size()];
    if Encode::encode(&request, &mut first.as_mut_slice()).is_err() {
        return;
    }

    // Re-decoding freshly encoded bytes must succeed.
    let Ok((reparsed, _)) = Request::decode(&first) else {
        panic!("failed to decode a message that was just encoded successfully");
    };

    // Encoding must be idempotent: re-encoding the reparsed message produces
    // the same bytes.
    let mut second = vec![0u8; reparsed.encoded_size()];
    Encode::encode(&reparsed, &mut second.as_mut_slice())
        .expect("failed to re-encode a decoded message");

    assert_eq!(
        first, second,
        "Roundtrip mismatch: encode(decode(encode(decode(data)))) != encode(decode(data))"
    );
});
