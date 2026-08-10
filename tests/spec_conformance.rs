//! Round-trips the message-flow example byte sequences printed in ISO 14229-1:2020.
//!
//! Every frame below is quoted from a numbered example table in the standard, which gives these
//! tests an oracle the rest of the suite does not have: the bytes come from the document rather
//! than from the crate.
//!
//! **What that does and does not buy.** It catches two things a round-trip against the crate's
//! own output cannot: a legal frame the crate *rejects* (a missing mandatory
//! `DTCFormatIdentifier`, a `MemorySelection` byte read from the wrong sub-functions), and an
//! encode/decode pair that disagree with each other (a `powerDownTime` written unconditionally,
//! so `51 01` re-encoded as `51 01 00`).
//!
//! It does **not** verify field *meaning*. If `encode` and `decode` share a misreading
//! symmetrically — two same-width adjacent fields transposed in both directions, say — the bytes
//! still round-trip and every test here passes. Only an assertion on the decoded *value* catches
//! that, which is the unit tests' job. Do not read a green run here as "the layout is right".
//!
//! This is an integration test, so it sees the crate as a downstream user does: anything it
//! needs must be reachable and constructible through the public API.

use uds_protocol::{Decode, Encode, Request, Response};

/// A frame quoted from the standard, with the table it came from.
struct Example {
    /// Clause and table the bytes are quoted from, used in failure messages.
    cite: &'static str,
    /// The complete application-layer frame, service identifier first.
    bytes: &'static [u8],
}

const REQUESTS: &[Example] = &[
    Example {
        cite: "Table 61 - CommunicationControl request (enableRxAndDisableTxWithEnhancedAddressInformation)",
        bytes: &[0x28, 0x04, 0x01, 0x00, 0x0A],
    },
    Example {
        cite: "Table 63 - CommunicationControl request (enableRxAndTxWithEnhancedAddressInformation)",
        bytes: &[0x28, 0x05, 0x01, 0x00, 0x0A],
    },
    Example {
        cite: "Table 124 - TesterPresent request example #1",
        bytes: &[0x3E, 0x00],
    },
    Example {
        cite: "Table 126 - TesterPresent request example #2 (suppressPosRspMsgIndicationBit = TRUE)",
        bytes: &[0x3E, 0x80],
    },
    Example {
        cite: "Table 133 - ControlDTCSetting request example #1 (DTCSettingType = off)",
        bytes: &[0x85, 0x02],
    },
    Example {
        cite: "Table 135 - ControlDTCSetting request example #2 (DTCSettingType = on)",
        bytes: &[0x85, 0x01],
    },
    Example {
        cite: "Table 300 - ClearDiagnosticInformation request example #1 (groupOfDTC = FFFF33, no MemorySelection)",
        bytes: &[0x14, 0xFF, 0xFF, 0x33],
    },
    Example {
        cite: "Table 354 - ReadDTCInformation request example #5 (reportDTCSnapshotRecordByDTCNumber)",
        bytes: &[0x19, 0x04, 0x12, 0x34, 0x56, 0x02],
    },
    Example {
        cite: "Table 361 - ReadDTCInformation request example #7 (reportDTCExtDataRecordByDTCNumber)",
        bytes: &[0x19, 0x06, 0x12, 0x34, 0x56, 0xFF],
    },
    Example {
        cite: "Table 404 - ReadDataByIdentifier request example #1 step #1",
        bytes: &[0x22, 0x9B, 0x00],
    },
    Example {
        cite: "Table 282 - WriteDataByIdentifier request example #1 (VIN)",
        bytes: &[
            0x2E, 0xF1, 0x90, 0x57, 0x30, 0x4C, 0x30, 0x30, 0x30, 0x30, 0x34, 0x33, 0x4D, 0x42,
            0x35, 0x34, 0x31, 0x33, 0x32, 0x36,
        ],
    },
    Example {
        cite: "Table 431 - RoutineControl request example #1 (startRoutine)",
        bytes: &[0x31, 0x01, 0x02, 0x01],
    },
    Example {
        cite: "Table 433 - RoutineControl request example #2 (stopRoutine)",
        bytes: &[0x31, 0x02, 0x02, 0x01],
    },
    Example {
        cite: "Table 462 - RequestDownload request (addressAndLengthFormatIdentifier = 0x33)",
        bytes: &[0x34, 0x11, 0x33, 0x60, 0x20, 0x00, 0x00, 0xFF, 0xFF],
    },
    Example {
        cite: "Table 466 - TransferData request (blockSequenceCounter = 5, no data)",
        bytes: &[0x36, 0x05],
    },
    Example {
        cite: "Table 468 - RequestTransferExit request",
        bytes: &[0x37],
    },
    // Table 486: modeOfOperation = AddFile, filePathAndNameLength = 0x001E,
    // filePathAndName = "D:\mapdata\europe\germany1.yxz", dataFormatIdentifier = 0x11,
    // fileSizeParameterLength = 2, fileSizeUnCompressed = 0xC350, fileSizeCompressed = 0x7530.
    Example {
        cite: "Table 486 - RequestFileTransfer request example (AddFile)",
        bytes: &[
            0x38, 0x01, 0x00, 0x1E, 0x44, 0x3A, 0x5C, 0x6D, 0x61, 0x70, 0x64, 0x61, 0x74, 0x61,
            0x5C, 0x65, 0x75, 0x72, 0x6F, 0x70, 0x65, 0x5C, 0x67, 0x65, 0x72, 0x6D, 0x61, 0x6E,
            0x79, 0x31, 0x2E, 0x79, 0x78, 0x7A, 0x11, 0x02, 0xC3, 0x50, 0x75, 0x30,
        ],
    },
    Example {
        cite: "Table 49 - SecurityAccess request example #1 step #2 (sendKey)",
        bytes: &[0x27, 0x02, 0xC9, 0xA9],
    },
];

const RESPONSES: &[Example] = &[
    Example {
        cite: "Table 32 - DiagnosticSessionControl positive response (P2 = 0x0032, P2* = 0x01F4)",
        bytes: &[0x50, 0x02, 0x00, 0x32, 0x01, 0xF4],
    },
    Example {
        cite: "Table 39 - ECUReset positive response example #1 (hardReset, no powerDownTime)",
        bytes: &[0x51, 0x01],
    },
    Example {
        cite: "Table 50 - SecurityAccess positive response example #1 step #2 (sendKey)",
        bytes: &[0x67, 0x02],
    },
    Example {
        cite: "Table 52 - SecurityAccess positive response example #2 step #2 (requestSeed)",
        bytes: &[0x67, 0x01, 0x00, 0x00],
    },
    Example {
        cite: "Table 60 - CommunicationControl positive response",
        bytes: &[0x68, 0x01],
    },
    Example {
        cite: "Table 62 - CommunicationControl positive response",
        bytes: &[0x68, 0x04],
    },
    Example {
        cite: "Table 125 - TesterPresent positive response example #1",
        bytes: &[0x7E, 0x00],
    },
    Example {
        cite: "Table 136 - ControlDTCSetting positive response example #2",
        bytes: &[0xC5, 0x01],
    },
    Example {
        cite: "Table 341 - ReadDTCInformation positive response example #1 (reportNumberOfDTCByStatusMask)",
        bytes: &[0x59, 0x01, 0x2F, 0x01, 0x00, 0x01],
    },
    Example {
        cite: "Table 405 - ReadDataByIdentifier positive response example #1 step #1",
        bytes: &[0x62, 0x9B, 0x00, 0x0A],
    },
    Example {
        cite: "Table 283 - WriteDataByIdentifier positive response example #1",
        bytes: &[0x6E, 0xF1, 0x90],
    },
    Example {
        cite: "Table 432 - RoutineControl positive response example #1",
        bytes: &[0x71, 0x01, 0x02, 0x01, 0x32],
    },
    Example {
        cite: "Table 434 - RoutineControl positive response example #2",
        bytes: &[0x71, 0x02, 0x02, 0x01, 0x30],
    },
    Example {
        cite: "Table 463 - RequestDownload positive response (maxNumberOfBlockLength = 0x0081)",
        bytes: &[0x74, 0x20, 0x00, 0x81],
    },
    Example {
        cite: "Table 465 - TransferData positive response",
        bytes: &[0x76, 0x01],
    },
    Example {
        cite: "Table 467 - TransferData positive response",
        bytes: &[0x76, 0x05],
    },
    Example {
        cite: "Table 469 - RequestTransferExit positive response",
        bytes: &[0x77],
    },
    Example {
        cite: "Table 487 - RequestFileTransfer positive response example (AddFile)",
        bytes: &[0x78, 0x01, 0x02, 0xC3, 0x50, 0x11],
    },
];

/// Largest example frame, plus room to catch an encoder that writes too much.
const BUF: usize = 64;

#[test]
fn every_spec_request_example_decodes_and_re_encodes_unchanged() {
    for Example { cite, bytes } in REQUESTS {
        let req = Request::decode_exact(bytes)
            .unwrap_or_else(|e| panic!("{cite}: decode of {bytes:02X?} failed: {e:?}"));
        // `Request::Other` carries the SID and payload verbatim, so it round-trips perfectly.
        // Without this, a frame for a service the crate does not model would pass silently and
        // this suite would report coverage it does not have.
        assert!(
            !matches!(req, Request::Other { .. }),
            "{cite}: decoded to Request::Other, so this service is not actually modeled"
        );
        let mut buf = [0u8; BUF];
        let written = req
            .encode_to_slice(&mut buf)
            .unwrap_or_else(|e| panic!("{cite}: encode failed: {e:?}"));
        assert_eq!(&buf[..written], *bytes, "{cite}: re-encoded bytes differ");
    }
}

#[test]
fn every_spec_response_example_decodes_and_re_encodes_unchanged() {
    for Example { cite, bytes } in RESPONSES {
        let resp = Response::decode_exact(bytes)
            .unwrap_or_else(|e| panic!("{cite}: decode of {bytes:02X?} failed: {e:?}"));
        // Same trap as on the request side: `Response::Other` round-trips any SID unchanged.
        assert!(
            !matches!(resp, Response::Other { .. }),
            "{cite}: decoded to Response::Other, so this service is not actually modeled"
        );
        let mut buf = [0u8; BUF];
        let written = resp
            .encode_to_slice(&mut buf)
            .unwrap_or_else(|e| panic!("{cite}: encode failed: {e:?}"));
        assert_eq!(&buf[..written], *bytes, "{cite}: re-encoded bytes differ");
    }
}

#[test]
fn the_set_of_services_with_spec_example_coverage_is_pinned() {
    // A tripwire on this file's own fixtures, not a property of the crate: it reads the two
    // const arrays above, so no production change can make it fail. Its only job is to force a
    // deliberate edit here when a service gains or loses spec-example coverage. The previous
    // name claimed it checked coverage of "every service the crate models", which it does not
    // and cannot -- the crate models 16 request services and 13 have frames below.
    let mut sids: Vec<u8> = REQUESTS.iter().map(|e| e.bytes[0]).collect();
    sids.sort_unstable();
    sids.dedup();
    assert_eq!(
        sids,
        vec![
            0x14, 0x19, 0x22, 0x27, 0x28, 0x2E, 0x31, 0x34, 0x36, 0x37, 0x38, 0x3E, 0x85
        ],
        "request-side spec-example coverage changed"
    );

    // Where a modeled service has no request frame here, the reason is that its example
    // table's byte-value column is not machine-readable in the markdown conversion, NOT that
    // the standard omits the example. An earlier version of this comment claimed the latter
    // about 0x10, 0x11 and 0x27, and that was simply wrong: Table 31 (DiagnosticSessionControl),
    // Table 38 (ECUReset) and Tables 47/49/51 (SecurityAccess) are all request byte tables.
    //
    // In Tables 31, 38 and 47 the conversion merged the byte value into the description cell
    // ("ECUReset Request SID 11 16"), so the values can only be recovered by cross-referencing
    // the parameter-definition tables — that is inference, not quotation, and quotation is the
    // whole point of this suite. Table 49 survived the conversion intact and is included above.
    // Table 472 (RequestUpload) is the same story: bytes #7 and #8 of its memorySize are not
    // legible, so 0x35 has no frame here despite being fully modeled.
    //
    // This is the same class of extraction hazard as the two-bytes-in-one-cell rows noted on
    // Tables 486 and 487. Verify against the PDF before adding a frame, never against a guess.
    let mut response_sids: Vec<u8> = RESPONSES.iter().map(|e| e.bytes[0]).collect();
    response_sids.sort_unstable();
    response_sids.dedup();
    assert_eq!(
        response_sids,
        vec![
            0x50, 0x51, 0x59, 0x62, 0x67, 0x68, 0x6E, 0x71, 0x74, 0x76, 0x77, 0x78, 0x7E, 0xC5
        ],
        "response-side spec-example coverage changed"
    );
}
