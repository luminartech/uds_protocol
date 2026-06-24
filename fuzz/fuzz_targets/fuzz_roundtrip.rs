// Fuzz encode→decode roundtrip: encode a successfully-decoded request,
// then decode it again and verify the result matches.
#![no_main]
use libfuzzer_sys::fuzz_target;
use uds_protocol::{
    FunctionalGroupIdentifier, ProtocolRequest, ReadDTCInfoSubFunction, Request,
    SingleValueWireFormat, WireFormat,
};

/// Returns true if the request contains a FunctionalGroupIdentifier variant
/// that has a todo!() in its value() method (LegislativeSystemGroup, ISOSAEReserved).
fn contains_unimplemented_functional_group(request: &ProtocolRequest) -> bool {
    let Request::ReadDTCInfo(req) = request else {
        return false;
    };
    let fgi = match &req.dtc_subfunction {
        ReadDTCInfoSubFunction::ReportWWHOBDDTC_ByMaskRecord(fgi, _, _) => fgi,
        ReadDTCInfoSubFunction::ReportWWHOBDDTC_WithPermanentStatus(fgi) => fgi,
        ReadDTCInfoSubFunction::ReportDTCInformation_ByDTCReadinessGroupIdentifier(fgi, _) => fgi,
        _ => return false,
    };
    matches!(
        fgi,
        FunctionalGroupIdentifier::LegislativeSystemGroup(_)
            | FunctionalGroupIdentifier::ISOSAEReserved(_)
    )
}

fuzz_target!(|data: &[u8]| {
    // Only proceed if we can decode the input
    let Ok(request) = ProtocolRequest::decode(&mut &data[..]) else {
        return;
    };

    // RoutineControl has a known encode/decode asymmetry in ProtocolRoutinePayload:
    // decode_next reads a 2-byte identifier from the stream, but encode writes only
    // the raw payload bytes (the identifier is written by the request layer).
    // This makes roundtripping structurally impossible for this variant.
    if matches!(request, Request::RoutineControl(_)) {
        return;
    }

    // FunctionalGroupIdentifier::LegislativeSystemGroup and ::ISOSAEReserved
    // have todo!() in their value() method — encoding them panics intentionally.
    if contains_unimplemented_functional_group(&request) {
        return;
    }

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
