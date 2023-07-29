use std::collections::{HashMap, HashSet};

use candid::{CandidType, Nat, Principal, Func, Encode, Decode};
use ic_stable_structures::{Storable, BoundedStorable, StableBTreeMap, StableVec};
use serde::{Deserialize, Serialize};

use crate::memory::{StableMemory, init_content_stable_data};

pub type ChunkID = u32;
pub type Blob = Vec<u8>;

#[derive(CandidType, Clone, Deserialize, Serialize)]
pub struct AssetChunk {
    pub checksum: u32,
    pub content: Blob,
    pub created_at: u64,
    pub id: u32,
    pub order: u32,
    pub owner: Principal,
}

impl Storable for AssetChunk{
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(&self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for AssetChunk{
    const IS_FIXED_SIZE: bool = false;
    const MAX_SIZE: u32 = 2100000;
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum ContentEncoding {
    Identity,
    GZIP,
}

impl Storable for ContentEncoding{
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(&self).unwrap())
    }
}

impl BoundedStorable for ContentEncoding{
    const IS_FIXED_SIZE: bool = true;
    const MAX_SIZE: u32 = 1;
}

#[derive(CandidType, Clone, Deserialize, Serialize)]
pub struct AssetProperties {
    pub content_encoding: ContentEncoding,
    pub content_type: String,
    pub filename: String,
    pub checksum: u32,
}

impl Storable for AssetProperties{
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(&self).unwrap())
    }
}

impl BoundedStorable for AssetProperties{
    const IS_FIXED_SIZE: bool = false;
    const MAX_SIZE: u32 = 100;
}

#[derive(CandidType, Deserialize)]
pub struct Content(pub Vec<u8>);

impl Storable for Content{
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Content{
    const IS_FIXED_SIZE: bool = false;
    const MAX_SIZE: u32 = 2 * 1024 * 1023 + 100;
}

#[derive(Deserialize, Serialize)]
pub struct Asset {
    #[serde(skip, default = "init_content_stable_data")]
    pub content: StableBTreeMap<u32, Content, StableMemory>,
    pub canister_id: Principal,
    pub chunk_size: u32,
    pub content_encoding: ContentEncoding,
    pub content_type: String,
    pub filename: String,
    pub content_size: u32,
    pub created_at: u64,
    pub id: u128,
    pub owner: Principal,
    pub url: String,
}

impl Storable for Asset{
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref()).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes).unwrap();
        std::borrow::Cow::Owned(bytes)
    }
}

impl BoundedStorable for Asset{
    const IS_FIXED_SIZE: bool = false;
    /// 200mb, tried out with bigger amount throws out error
    const MAX_SIZE: u32 = 200 * 1024 * 1024;
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
    pub id: u128,
    pub owner: Principal,
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
    pub body: Vec<u8>,
    pub streaming_strategy: Option<StreamingStrategy>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CreateStrategyArgs{
    pub asset_id: u128,
    pub chunk_index: u32,
    pub data_chunks_size: Nat,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct StreamingCallbackToken{
    pub asset_id: u128,
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
    pub body: Vec<u8>,
    pub token: Option<StreamingCallbackToken>,
}