use super::UdsServiceType;

pub struct UdsResponse {
    pub service: UdsServiceType,
    pub data: Vec<u8>,
}
