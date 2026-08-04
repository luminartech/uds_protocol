//! Module for making and handling UDS Requests
use crate::{
    Decode, Encode, Error, Incomplete, NegativeResponseCode,
    services::{
        ClearDiagnosticInfoRequest, CommunicationControlRequest, ControlDtcSettingRequest,
        DiagnosticSessionControlRequest, EcuResetRequest, ReadDataByIdentifierRequest,
        ReadDtcInfoRequest, RequestDownloadRequest, RequestFileTransferRequest,
        RequestTransferExitRequest, RequestUploadRequest, RoutineControlRequest,
        SecurityAccessRequest, TesterPresentRequest, TransferDataRequest,
        WriteDataByIdentifierRequest,
    },
    shared::split_sprmib,
};
use automotive_wire_codec::{write_all, write_u8};

use super::service::UdsServiceType;

/// Zero-copy parsed request. Borrows from the wire buffer.
///
/// Variable-length payloads are stored as raw `&'a [u8]` slices that can be
/// further parsed on demand.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Request<'a> {
    /// Clear diagnostic information request.
    ClearDiagnosticInfo(ClearDiagnosticInfoRequest),
    /// Communication control request.
    CommunicationControl(CommunicationControlRequest),
    /// Control DTC settings request.
    ControlDtcSetting(ControlDtcSettingRequest<'a>),
    /// Diagnostic session control request.
    DiagnosticSessionControl(DiagnosticSessionControlRequest),
    /// ECU reset request.
    EcuReset(EcuResetRequest),
    /// Read data by identifier request.
    ReadDataByIdentifier(ReadDataByIdentifierRequest<'a>),
    /// Read DTC information request.
    ReadDtcInfo(ReadDtcInfoRequest),
    /// Request download.
    RequestDownload(RequestDownloadRequest),
    /// Request file transfer.
    RequestFileTransfer(RequestFileTransferRequest<'a>),
    /// Request transfer exit.
    RequestTransferExit(RequestTransferExitRequest<'a>),
    /// Request upload.
    RequestUpload(RequestUploadRequest),
    /// Routine control request.
    RoutineControl(RoutineControlRequest<'a>),
    /// Security access request.
    SecurityAccess(SecurityAccessRequest<'a>),
    /// Tester present request.
    TesterPresent(TesterPresentRequest),
    /// Transfer data request.
    TransferData(TransferDataRequest<'a>),
    /// Write data by identifier request.
    WriteDataByIdentifier(WriteDataByIdentifierRequest<'a>),
    /// A known-but-unmodeled (or unrecognized) service. Carries the raw service byte and
    /// the raw payload bytes following the service identifier, for pass-through.
    ///
    /// Re-encoding is lossless for every service byte: the raw `sid` is echoed verbatim.
    Other {
        /// The raw service identifier byte from the wire.
        sid: u8,
        /// Raw payload bytes after the service byte.
        data: &'a [u8],
    },
}

/// # Remainder
///
/// The returned remainder is **always empty**. A UDS frame is not self-delimiting — its length
/// comes from the transport (ISO-TP, `DoIP`, ...), not from the message — so one buffer is exactly
/// one frame, and every payload is decoded with `decode_exact`. This means `decode` behaves as
/// `decode_exact` despite the streaming shape of the [`Decode`] contract: do **not** feed it
/// concatenated frames expecting it to consume one at a time, and note that
/// [`DecodeIter`](crate::DecodeIter) over such a buffer would treat the whole thing as a single
/// frame. Split frames at the transport layer before calling this.
impl<'a> Decode<'a> for Request<'a> {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        let service = UdsServiceType::from_request_sid(buf[0]);
        let payload = &buf[1..];

        let request = match service {
            UdsServiceType::ClearDiagnosticInfo => Self::ClearDiagnosticInfo(
                <ClearDiagnosticInfoRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::CommunicationControl => Self::CommunicationControl(
                <CommunicationControlRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::ControlDtcSetting => Self::ControlDtcSetting(
                <ControlDtcSettingRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::DiagnosticSessionControl => Self::DiagnosticSessionControl(
                <DiagnosticSessionControlRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::EcuReset => {
                Self::EcuReset(<EcuResetRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::ReadDataByIdentifier => Self::ReadDataByIdentifier(
                <ReadDataByIdentifierRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::ReadDtcInfo => {
                Self::ReadDtcInfo(<ReadDtcInfoRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::RequestDownload => {
                Self::RequestDownload(<RequestDownloadRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::RequestFileTransfer => Self::RequestFileTransfer(
                <RequestFileTransferRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit(
                <RequestTransferExitRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::RequestUpload => {
                Self::RequestUpload(<RequestUploadRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::RoutineControl => {
                Self::RoutineControl(<RoutineControlRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::SecurityAccess => {
                Self::SecurityAccess(<SecurityAccessRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::TesterPresent => {
                Self::TesterPresent(<TesterPresentRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::TransferData => {
                Self::TransferData(<TransferDataRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::WriteDataByIdentifier => Self::WriteDataByIdentifier(
                <WriteDataByIdentifierRequest as Decode>::decode_exact(payload)?,
            ),
            _ => Self::Other {
                sid: buf[0],
                data: payload,
            },
        };
        Ok((request, &[]))
    }
}

impl Encode for Request<'_> {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let sid = match self {
            Self::Other { sid, .. } => *sid,
            other => other.service().to_request_sid(),
        };
        let sid_len = write_u8(writer, sid).map_err(Error::io)?;
        let payload = match self {
            Self::ClearDiagnosticInfo(req) => req.encode(writer)?,
            Self::CommunicationControl(req) => req.encode(writer)?,
            Self::ControlDtcSetting(req) => req.encode(writer)?,
            Self::DiagnosticSessionControl(req) => req.encode(writer)?,
            Self::EcuReset(req) => req.encode(writer)?,
            Self::ReadDataByIdentifier(req) => req.encode(writer)?,
            Self::ReadDtcInfo(req) => req.encode(writer)?,
            Self::WriteDataByIdentifier(req) => req.encode(writer)?,
            Self::RequestDownload(req) => req.encode(writer)?,
            Self::RequestFileTransfer(req) => req.encode(writer)?,
            Self::RequestTransferExit(req) => req.encode(writer)?,
            Self::RequestUpload(req) => req.encode(writer)?,
            Self::Other { data, .. } => write_all(writer, data).map_err(Error::io)?,
            Self::RoutineControl(req) => req.encode(writer)?,
            Self::SecurityAccess(req) => req.encode(writer)?,
            Self::TesterPresent(req) => req.encode(writer)?,
            Self::TransferData(req) => req.encode(writer)?,
        };
        Ok(sid_len + payload)
    }
}

impl Request<'_> {
    /// Whether the positive response for this request is suppressed (SPRMIB).
    ///
    /// Only services ISO 14229-1 gives a sub-function can suppress a positive response, because
    /// the suppressPosRspMsgIndicationBit is bit 7 of that sub-function byte. For a service with
    /// no sub-function the answer is a definite `Some(false)`.
    ///
    /// # `None` means the question has no answer, not that it was not asked
    ///
    /// Returns `None` in exactly three situations, all of which are genuinely unanswerable
    /// rather than merely unmodeled:
    ///
    /// - The service identifier is not one ISO 14229-1 assigns — the vendor-specific case. The
    ///   crate has no basis for an answer, and a caller who needs one has to supply it from the
    ///   application that originated the request.
    /// - The service has a sub-function but the payload is empty, so the byte holding the SPRMIB
    ///   is not present. Such a frame is malformed for its service; either way there is no bit
    ///   to report.
    /// - The service identifier is `0x83`, whose service the 2020 edition withdrew. See
    ///   [`UdsServiceType::has_sub_function`] for why that answer is deferred to the caller
    ///   rather than taken from the 2013 edition.
    ///
    /// A service this crate enumerates but does not model still gets a real answer, because
    /// whether it carries a sub-function is a fact of the standard: see
    /// [`UdsServiceType::has_sub_function`]. So a `0x2C`
    /// [`DynamicallyDefineDataIdentifier`](UdsServiceType::DynamicallyDefineDataIdentifier)
    /// request reports its SPRMIB even though it decodes to [`Request::Other`].
    ///
    /// # Why this is `Option` when [`Request::allowed_nack_codes`] is not
    ///
    /// `allowed_nack_codes` reports "unknown" as an empty slice and documents it, which works
    /// because no service has an empty table of listed codes — the sentinel cannot be confused
    /// with a real answer. `false` has no such property: fourteen services genuinely have no
    /// sub-function, so a bare `bool` cannot distinguish them from a service the crate knows
    /// nothing about. That distinction has teeth at a layer boundary. ISO 14229-2 clause 10.3
    /// gates `tP3_Client_Phys` on this bit, and whether a response is expected decides whether
    /// `tP_Client` starts at all — so answering `false` for a fire-and-forget vendor request
    /// costs three transmissions of a request nobody wanted answered, per Table 9 retries.
    #[must_use]
    pub fn is_positive_response_suppressed(&self) -> Option<bool> {
        match self {
            Self::CommunicationControl(req) => Some(req.suppress_positive_response),
            Self::ControlDtcSetting(req) => Some(req.suppress_positive_response),
            Self::DiagnosticSessionControl(req) => Some(req.suppress_positive_response),
            Self::EcuReset(req) => Some(req.suppress_positive_response),
            Self::ReadDtcInfo(req) => Some(req.suppress_positive_response),
            Self::RoutineControl(req) => Some(req.suppress_positive_response),
            Self::SecurityAccess(req) => Some(req.suppress_positive_response),
            Self::TesterPresent(req) => Some(req.suppress_positive_response),
            // The modeled services ISO gives no sub-function. Spelled out rather than folded
            // into a wildcard: a wildcard is what made every unrecognized service silently
            // report `false`, and it would do the same to the next variant added here.
            Self::ClearDiagnosticInfo(_)
            | Self::ReadDataByIdentifier(_)
            | Self::RequestDownload(_)
            | Self::RequestFileTransfer(_)
            | Self::RequestTransferExit(_)
            | Self::RequestUpload(_)
            | Self::TransferData(_)
            | Self::WriteDataByIdentifier(_) => Some(false),
            Self::Other { data, .. } => {
                if self.service().has_sub_function()? {
                    data.first().copied().map(|byte| split_sprmib(byte).0)
                } else {
                    Some(false)
                }
            }
        }
    }

    /// The [`NegativeResponseCode`]s ISO 14229-1 **lists** for this request's service.
    ///
    /// Each request type also exposes this as an associated function (e.g.
    /// [`EcuResetRequest::allowed_nack_codes`]); this dispatches to it for a decoded
    /// [`Request`], so a server does not have to re-match every variant to reach it.
    ///
    /// # This is a floor, not a ceiling — do not use it as a validation whitelist
    ///
    /// Clause 9.4: the Annex A.1 codes "shall be used **in addition to** the negative response
    /// codes specified in each service description", and A.1 itself says a server "may also
    /// utilise additional and applicable negative response codes … as defined by the vehicle
    /// manufacturer". A.1 deliberately keeps the generally-supported codes out of the
    /// per-service tables and says so per code — including `0x78`
    /// [`RequestCorrectlyReceivedResponsePending`](NegativeResponseCode::RequestCorrectlyReceivedResponsePending),
    /// which appears in **none** of these tables and is one of the most common codes in real
    /// traffic. A client that rejected any NRC absent from this slice would reject every
    /// `ResponsePending` it ever saw.
    ///
    /// Read it as "the codes the standard tabulates for this service", which is useful for
    /// building a tester UI or a conformance report, and not as the set a server may send.
    ///
    /// Returns an empty slice for [`Request::Other`], which covers services the crate does not
    /// model. That means "the NRC set is unknown", not "no codes apply" — consult ISO 14229-1
    /// for those services.
    #[must_use]
    pub fn allowed_nack_codes(&self) -> &'static [NegativeResponseCode] {
        match self {
            Self::ClearDiagnosticInfo(_) => ClearDiagnosticInfoRequest::allowed_nack_codes(),
            Self::CommunicationControl(_) => CommunicationControlRequest::allowed_nack_codes(),
            Self::ControlDtcSetting(_) => ControlDtcSettingRequest::allowed_nack_codes(),
            Self::DiagnosticSessionControl(_) => {
                DiagnosticSessionControlRequest::allowed_nack_codes()
            }
            Self::EcuReset(_) => EcuResetRequest::allowed_nack_codes(),
            Self::ReadDataByIdentifier(_) => ReadDataByIdentifierRequest::allowed_nack_codes(),
            Self::ReadDtcInfo(_) => ReadDtcInfoRequest::allowed_nack_codes(),
            Self::RequestDownload(_) => RequestDownloadRequest::allowed_nack_codes(),
            Self::RequestFileTransfer(_) => RequestFileTransferRequest::allowed_nack_codes(),
            Self::RequestTransferExit(_) => RequestTransferExitRequest::allowed_nack_codes(),
            Self::RequestUpload(_) => RequestUploadRequest::allowed_nack_codes(),
            Self::RoutineControl(_) => RoutineControlRequest::allowed_nack_codes(),
            Self::SecurityAccess(_) => SecurityAccessRequest::allowed_nack_codes(),
            Self::TesterPresent(_) => TesterPresentRequest::allowed_nack_codes(),
            Self::TransferData(_) => TransferDataRequest::allowed_nack_codes(),
            Self::WriteDataByIdentifier(_) => WriteDataByIdentifierRequest::allowed_nack_codes(),
            Self::Other { .. } => &[],
        }
    }

    /// Returns the [`UdsServiceType`] corresponding to this request variant.
    #[must_use]
    pub fn service(&self) -> UdsServiceType {
        match self {
            Self::ClearDiagnosticInfo(_) => UdsServiceType::ClearDiagnosticInfo,
            Self::CommunicationControl(_) => UdsServiceType::CommunicationControl,
            Self::ControlDtcSetting(_) => UdsServiceType::ControlDtcSetting,
            Self::DiagnosticSessionControl(_) => UdsServiceType::DiagnosticSessionControl,
            Self::EcuReset(_) => UdsServiceType::EcuReset,
            Self::ReadDataByIdentifier(_) => UdsServiceType::ReadDataByIdentifier,
            Self::ReadDtcInfo(_) => UdsServiceType::ReadDtcInfo,
            Self::RequestDownload(_) => UdsServiceType::RequestDownload,
            Self::RequestFileTransfer(_) => UdsServiceType::RequestFileTransfer,
            Self::RequestTransferExit(_) => UdsServiceType::RequestTransferExit,
            Self::RequestUpload(_) => UdsServiceType::RequestUpload,
            Self::RoutineControl(_) => UdsServiceType::RoutineControl,
            Self::SecurityAccess(_) => UdsServiceType::SecurityAccess,
            Self::TesterPresent(_) => UdsServiceType::TesterPresent,
            Self::TransferData(_) => UdsServiceType::TransferData,
            Self::WriteDataByIdentifier(_) => UdsServiceType::WriteDataByIdentifier,
            Self::Other { sid, .. } => UdsServiceType::from_request_sid(*sid),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ResetType, service::UdsServiceType};

    #[test]
    fn decode_rejects_trailing_bytes() {
        // ECU reset is a fixed 1-byte payload; an extra trailing byte is a
        // malformed frame and must be rejected rather than silently dropped.
        let mut frame = [0u8; 3];
        frame[0] = UdsServiceType::EcuReset.to_request_sid();
        frame[1] = u8::from(ResetType::HardReset);
        frame[2] = 0xAA; // trailing junk
        let result = Request::decode(&frame);
        assert!(matches!(result, Err(Error::TrailingBytes(_))));
    }

    #[test]
    fn suppression_forwards_to_inner_request() {
        let suppressed = Request::EcuReset(EcuResetRequest::new(true, ResetType::HardReset));
        assert_eq!(suppressed.is_positive_response_suppressed(), Some(true));

        let not_suppressed = Request::EcuReset(EcuResetRequest::new(false, ResetType::HardReset));
        assert_eq!(
            not_suppressed.is_positive_response_suppressed(),
            Some(false)
        );
    }

    #[test]
    fn suppression_is_read_from_the_sub_function_of_an_unmodeled_service() {
        // 0x2C DynamicallyDefineDataIdentifier is enumerated but unmodeled, so it decodes to
        // `Other`. It does have a sub-function, so bit 7 of the first payload byte is its
        // SPRMIB and the crate can answer even though it does not model the payload.
        // Sub-function 0x01 defineByIdentifier, with and without the bit.
        let (suppressed, _) = Request::decode(&[0x2C, 0x81, 0xF3, 0x00]).unwrap();
        assert!(matches!(suppressed, Request::Other { .. }));
        assert_eq!(suppressed.is_positive_response_suppressed(), Some(true));

        let (not_suppressed, _) = Request::decode(&[0x2C, 0x01, 0xF3, 0x00]).unwrap();
        assert_eq!(
            not_suppressed.is_positive_response_suppressed(),
            Some(false)
        );
    }

    #[test]
    fn an_unmodeled_service_without_a_sub_function_is_never_suppressed() {
        // 0x23 ReadMemoryByAddress is enumerated but unmodeled, and ISO gives it no
        // sub-function -- so there is no SPRMIB anywhere in the frame and the answer is a
        // definite `Some(false)`, whatever the payload bytes happen to be. 0xAA has bit 7 set,
        // which would read as "suppressed" if the payload were mistaken for a sub-function.
        let (req, _) = Request::decode(&[0x23, 0xAA, 0xBB]).unwrap();
        assert!(matches!(req, Request::Other { .. }));
        assert_eq!(req.is_positive_response_suppressed(), Some(false));
    }

    #[test]
    fn suppression_is_unknown_for_a_vendor_specific_service() {
        // 0x40 is not in the ISO request table, so it is presumably vendor-specific and the
        // crate has no basis for an answer. This is the case the `Option` exists for: reporting
        // `false` here made a fire-and-forget vendor request look response-expected, so a
        // session layer started tP_Client, timed out, and retried it twice.
        let (req, _) = Request::decode(&[0x40, 0xAA, 0xBB]).unwrap();
        assert!(matches!(req, Request::Other { .. }));
        assert_eq!(req.is_positive_response_suppressed(), None);
    }

    #[test]
    fn suppression_is_unknown_when_the_sub_function_byte_is_absent() {
        // 0x2C has a sub-function, but this frame carries no payload -- so the byte the SPRMIB
        // lives in is not on the wire. The frame is malformed for the service; either way there
        // is no bit to report, so the answer is unknown rather than `Some(false)`.
        let (req, _) = Request::decode(&[0x2C]).unwrap();
        assert!(matches!(req, Request::Other { data, .. } if data.is_empty()));
        assert_eq!(req.is_positive_response_suppressed(), None);
    }

    #[test]
    fn write_data_by_identifier_request_roundtrips() {
        // SID 0x2E, DID 0xF190, one data byte 0x01
        let wire = [0x2E, 0xF1, 0x90, 0x01];
        let (req, rest) = Request::decode(&wire).unwrap();
        assert!(rest.is_empty());
        assert!(matches!(req, Request::WriteDataByIdentifier(_)));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn routine_control_request_roundtrips_with_suppress_bit() {
        // SID 0x31, sub 0x81 (StartRoutine + SPRMIB), RID 0xFF00, param 0xAA
        let wire = [0x31, 0x81, 0xFF, 0x00, 0xAA];
        let (req, rest) = Request::decode(&wire).unwrap();
        assert!(rest.is_empty());
        assert_eq!(req.is_positive_response_suppressed(), Some(true));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    /// One minimal-but-valid frame per modeled service, paired with the NRC table that service
    /// must dispatch to.
    ///
    /// Shared by `allowed_nack_codes_dispatches_for_every_modeled_variant` and
    /// `every_modeled_variant_agrees_with_the_sub_function_table` so the two cannot drift apart.
    /// A function rather than a `const`, because `allowed_nack_codes` is not a `const fn`.
    ///
    /// Every service that has a sub-function has bit 7 of that byte SET. That is load-bearing
    /// for the sub-function assertion: with the bit clear every service answers `Some(false)`,
    /// so a flipped table entry would go unnoticed. Setting it changes neither which service a
    /// frame decodes to nor its NRC table, so the NRC assertions are indifferent to it.
    ///
    /// Two pairs are deliberately indistinguishable, and that is correct rather than a gap: ISO
    /// gives `CommunicationControl` and `ControlDTCSetting` the same four codes, and Tables 444
    /// and 449 give `RequestDownload` and `RequestUpload` the same six. Swapping either pair's
    /// rows is unobservable because the answer is the same.
    fn modeled_frames() -> [(&'static [u8], &'static [NegativeResponseCode]); 16] {
        [
            // ClearDiagnosticInfo, no sub-function
            (
                &[0x14, 0xFF, 0xFF, 0xFF, 0x00],
                ClearDiagnosticInfoRequest::allowed_nack_codes(),
            ),
            // CommunicationControl, EnableRxAndTx + SPRMIB
            (
                &[0x28, 0x80, 0x01],
                CommunicationControlRequest::allowed_nack_codes(),
            ),
            // ControlDtcSetting, On + SPRMIB
            (
                &[0x85, 0x81],
                ControlDtcSettingRequest::allowed_nack_codes(),
            ),
            // DiagnosticSessionControl, DefaultSession + SPRMIB
            (
                &[0x10, 0x81],
                DiagnosticSessionControlRequest::allowed_nack_codes(),
            ),
            // EcuReset, HardReset + SPRMIB
            (&[0x11, 0x81], EcuResetRequest::allowed_nack_codes()),
            // ReadDataByIdentifier, no sub-function
            (
                &[0x22, 0xF1, 0x90],
                ReadDataByIdentifierRequest::allowed_nack_codes(),
            ),
            // ReadDtcInfo, ReportDtcByStatusMask + SPRMIB
            (
                &[0x19, 0x82, 0xFF],
                ReadDtcInfoRequest::allowed_nack_codes(),
            ),
            // RequestDownload, no sub-function
            (
                &[0x34, 0x00, 0x12, 0xBE, 0xEF, 0x10],
                RequestDownloadRequest::allowed_nack_codes(),
            ),
            // RequestFileTransfer, no sub-function
            (
                &[0x38, 0x02, 0x00, 0x01, b'a'],
                RequestFileTransferRequest::allowed_nack_codes(),
            ),
            // RequestUpload, no sub-function
            (
                &[0x35, 0x00, 0x12, 0xBE, 0xEF, 0x10],
                RequestUploadRequest::allowed_nack_codes(),
            ),
            // RequestTransferExit, no sub-function
            (&[0x37], RequestTransferExitRequest::allowed_nack_codes()),
            // RoutineControl, StartRoutine + SPRMIB
            (
                &[0x31, 0x81, 0xFF, 0x00],
                RoutineControlRequest::allowed_nack_codes(),
            ),
            // SecurityAccess, RequestSeed + SPRMIB
            (
                &[0x27, 0x81, 0xAA],
                SecurityAccessRequest::allowed_nack_codes(),
            ),
            // TesterPresent, ZeroSubFunction + SPRMIB
            (&[0x3E, 0x80], TesterPresentRequest::allowed_nack_codes()),
            // TransferData, no sub-function
            (
                &[0x36, 0x01, 0xAA],
                TransferDataRequest::allowed_nack_codes(),
            ),
            // WriteDataByIdentifier, no sub-function
            (
                &[0x2E, 0xF1, 0x90, 0x01],
                WriteDataByIdentifierRequest::allowed_nack_codes(),
            ),
        ]
    }

    #[test]
    fn allowed_nack_codes_dispatches_for_every_modeled_variant() {
        // Every one of the 16 request types has an inherent `allowed_nack_codes()`, but
        // without this dispatcher a caller holding a *decoded* `Request` had to re-match all
        // 16 variants to reach it — on a `#[non_exhaustive]` enum they cannot match
        // exhaustively. Frames are minimal-but-valid for each service.
        //
        // Each frame is paired with the inherent table it must dispatch to. Asserting only
        // `!is_empty()` made the test vacuous: every modeled service has a non-empty table, so
        // any arm could return any *other* service's set and still pass.
        let frames = modeled_frames();
        for (frame, expected) in frames {
            let (req, _) = Request::decode(frame).unwrap_or_else(|e| {
                panic!("frame {frame:02X?} should decode, got {e:?}");
            });
            assert!(
                !matches!(req, Request::Other { .. }),
                "frame {frame:02X?} decoded to Other; the table needs updating"
            );
            assert_eq!(
                req.allowed_nack_codes(),
                expected,
                "{:?} dispatched to the wrong NRC table",
                req.service()
            );
        }
    }

    #[test]
    fn every_modeled_variant_agrees_with_the_sub_function_table() {
        // `has_sub_function` and the match in `is_positive_response_suppressed` are two copies
        // of one ISO fact, and nothing else stops them from drifting. This ties them together as
        // a biconditional: the table says a service has a sub-function exactly when the dispatch
        // reports a set SPRMIB for a frame that sets one.
        //
        // `modeled_frames` sets the SPRMIB on every service that has one, which is what makes
        // the equality below meaningful in both directions.
        for (frame, _) in modeled_frames() {
            let (req, _) = Request::decode(frame).unwrap_or_else(|e| {
                panic!("frame {frame:02X?} should decode, got {e:?}");
            });
            assert!(
                !matches!(req, Request::Other { .. }),
                "frame {frame:02X?} decoded to Other; the table needs updating"
            );
            let service = req.service();
            let suppressed = req.is_positive_response_suppressed();
            assert!(
                suppressed.is_some(),
                "{service:?} is modeled, so its SPRMIB is never unknown"
            );
            // The frames set the SPRMIB wherever the service has a sub-function to set it in, so
            // the two must be equal: `Some(true)` for the eight services that have one,
            // `Some(false)` for the eight that do not. Any table entry flipped either way breaks
            // this.
            assert_eq!(
                suppressed,
                service.has_sub_function(),
                "{service:?}: dispatch and sub-function table disagree"
            );
        }
    }

    #[test]
    fn allowed_nack_codes_agrees_with_the_inherent_method() {
        let (req, _) = Request::decode(&[0x11, 0x01]).unwrap();
        assert_eq!(
            req.allowed_nack_codes(),
            EcuResetRequest::allowed_nack_codes()
        );
    }

    #[test]
    fn allowed_nack_codes_is_empty_for_pass_through() {
        // 0x23 ReadMemoryByAddress is enumerated but unmodeled, so the crate has no NRC table
        // for it. An empty slice says "unknown", not "none apply".
        let (req, _) = Request::decode(&[0x23, 0xAA]).unwrap();
        assert!(matches!(req, Request::Other { .. }));
        assert!(req.allowed_nack_codes().is_empty());
    }

    #[test]
    fn unmodeled_service_decodes_to_other() {
        // 0x23 = ReadMemoryByAddress, enumerated but not modeled.
        let frame = [0x23, 0xAA, 0xBB];
        let (req, rest) = Request::decode(&frame).unwrap();
        assert!(rest.is_empty());
        match req {
            Request::Other { sid, data } => {
                assert_eq!(sid, 0x23);
                assert_eq!(data, &[0xAA, 0xBB]);
            }
            other => panic!("expected Other, got {other:?}"),
        }
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &frame);
    }

    #[test]
    fn unknown_request_byte_round_trips_losslessly() {
        // 0x40 is not in the ISO request table; it must survive a decode→encode round-trip.
        let frame = [0x40, 0xAA, 0xBB];
        let (req, rest) = Request::decode(&frame).unwrap();
        assert!(rest.is_empty());
        match req {
            Request::Other { sid, data } => {
                assert_eq!(sid, 0x40);
                assert_eq!(data, &[0xAA, 0xBB]);
            }
            other => panic!("expected Other, got {other:?}"),
        }
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &frame); // previously re-encoded as 0x7F
    }
}
