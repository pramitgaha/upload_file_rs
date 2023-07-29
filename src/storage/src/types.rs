use candid::{CandidType, Decode, Encode, Principal};
// use ic_stable_memory::{collections::SVec, derive::{StableType, AsFixedSizeBytes}};
use ic_stable_structures::{BoundedStorable, StableBTreeMap, Storable};

use crate::{
    chunk_handler::ChunkArg,
    memory::{init_asset_stable_data, init_chunk_stable_data, StableMemory},
};

#[derive(serde::Serialize, serde::Deserialize, CandidType)]
pub struct Chunk {
    pub content: Vec<u8>,
    pub owner: Principal,
    pub created_at: u64,
    pub order: u32,
    pub checksum: u32,
    pub id: u128,
}

// #[derive(StableType, AsFixedSizeBytes)]
// pub struct StableChunk{
//     pub content: SVec<u8>,
//     pub owner: Principal,
//     pub created_at: u64,
//     pub order: u32,
//     pub checksum: u32,
//     pub id: u128,
// }

impl From<(&Principal, u128, ChunkArg)> for Chunk {
    fn from((owner, id, args): (&Principal, u128, ChunkArg)) -> Self {
        let checksum = crc32fast::hash(&args.content);
        Self {
            content: args.content,
            owner: owner.clone(),
            created_at: ic_cdk::api::time(),
            order: args.order,
            checksum,
            id,
        }
    }
}

impl Storable for Chunk {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        // let mut bytes = vec![];
        // ciborium::ser::into_writer(&self, &mut bytes).unwrap();
        // std::borrow::Cow::Owned(bytes)
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        // ciborium::de::from_reader(bytes.as_ref()).unwrap()
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Chunk {
    const IS_FIXED_SIZE: bool = false;
    const MAX_SIZE: u32 = 3 * 1024 * 1024;
}

#[derive(CandidType, serde::Serialize, serde::Deserialize, Clone)]
pub enum ContentEncoding {
    Identity,
    GZIP,
}

impl Storable for ContentEncoding {
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

impl BoundedStorable for ContentEncoding {
    const IS_FIXED_SIZE: bool = true;
    const MAX_SIZE: u32 = 1;
}

#[derive(CandidType, serde::Serialize, serde::Deserialize)]
pub struct Asset {
    pub content: Vec<u8>,
    pub file_name: String,
    pub owner: Principal,
    pub content_encoding: ContentEncoding,
    pub url: String,
    pub id: u128,
    pub content_type: String,
}

impl Storable for Asset {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        // let mut bytes = vec![];
        // ciborium::ser::into_writer(&self, &mut bytes).unwrap();
        // std::borrow::Cow::Owned(bytes)
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        // ciborium::de::from_reader(bytes.as_ref()).unwrap()
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Asset {
    const IS_FIXED_SIZE: bool = false;
    const MAX_SIZE: u32 = 500 * 1024 * 1024;
}

#[derive(CandidType)]
pub struct AssetQuery {
    pub file_name: String,
    pub owner: Principal,
    pub content_encoding: ContentEncoding,
    pub url: String,
    pub id: u128,
    pub content_type: String,
}

impl From<&Asset> for AssetQuery {
    fn from(value: &Asset) -> Self {
        Self {
            file_name: value.file_name.clone(),
            owner: value.owner.clone(),
            content_encoding: value.content_encoding.clone(),
            url: value.url.clone(),
            id: value.id,
            content_type: value.content_type.clone(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct State {
    pub chunk_count: u128,
    #[serde(skip, default = "init_chunk_stable_data")]
    pub chunks: StableBTreeMap<u128, Chunk, StableMemory>,
    pub asset_count: u128,
    #[serde(skip, default = "init_asset_stable_data")]
    pub assets: StableBTreeMap<u128, Asset, StableMemory>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            chunk_count: 1,
            chunks: init_chunk_stable_data(),
            asset_count: 1,
            assets: init_asset_stable_data(),
        }
    }
}

impl State {
    pub fn get_chunk_id(&mut self) -> u128 {
        let id = self.chunk_count;
        self.chunk_count += 1;
        id
    }

    pub fn get_asset_id(&mut self) -> u128 {
        let id = self.asset_count;
        self.asset_count += 1;
        id
    }
}
