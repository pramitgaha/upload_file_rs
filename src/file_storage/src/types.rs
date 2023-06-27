use candid::{CandidType, Nat, Principal, Func};
use serde::Deserialize;

pub type ChunkID = u32;
pub type AssetID = String;
pub type Blob = Vec<u8>;

#[derive(CandidType, Clone, Deserialize)]
pub struct AssetChunk {
    pub checksum: u32,
    pub content: Blob,
    pub created_at: u64,
    pub file_name: String,
    pub id: u32,
    pub order: u32,
    pub owner: Principal,
}

#[derive(CandidType, Clone, Deserialize)]
pub enum ContentEncoding {
    Identity,
    GZIP,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct AssetProperties {
    pub content_encoding: ContentEncoding,
    pub content_type: String,
    pub filename: String,
    pub checksum: u32,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct Asset {
    pub content: Option<Vec<Blob>>,
    pub canister_id: Principal,
    pub chunk_size: u32,
    pub content_encoding: ContentEncoding,
    pub content_type: String,
    pub filename: String,
    pub content_size: u32,
    pub created_at: u64,
    pub id: String,
    pub owner: String,
    pub url: String,
}

#[derive(CandidType, Clone)]
pub struct AssetQuery {
    pub canister_id: Principal,
    pub chunk_size: u32,
    pub content_encoding: ContentEncoding,
    pub content_type: String,
    pub filename: String,
    pub content_size: u32,
    pub created_at: u64,
    pub id: String,
    pub owner: String,
    pub url: String,
}

impl From<&Asset> for AssetQuery {
    fn from(value: &Asset) -> Self {
        Self {
            canister_id: value.canister_id,
            chunk_size: value.chunk_size,
            content_encoding: value.content_encoding.clone(),
            content_type: value.content_type.clone(),
            filename: value.filename.clone(),
            content_size: value.content_size,
            created_at: value.created_at,
            id: value.id.clone(),
            owner: value.owner.clone(),
            url: value.url.clone(),
        }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct HeaderField(pub String, pub String);

#[derive(CandidType, Deserialize, Clone)]
pub struct HttpRequest{
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    pub body: Blob,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct HttpResponse{
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: Blob,
    pub streaming_strategy: Option<StreamingStrategy>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CreateStrategyArgs{
    pub asset_id: String,
    pub chunk_index: u32,
    pub data_chunks_size: Nat,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct StreamingCallbackToken{
    pub asset_id: String,
    pub chunk_index: u32,
    pub content_encoding: String,
}

#[derive(CandidType, Deserialize, Clone)]
pub enum StreamingStrategy{
    Callback{
        token: StreamingCallbackToken,
        callback: Func,
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct StreamingCallbackHttpResponse{
    pub body: Blob,
    pub token: Option<StreamingCallbackToken>,
}