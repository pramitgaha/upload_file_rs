use std::{cell::RefCell, collections::HashMap};

use candid::{candid_method, Nat, Func};
use ic_cdk_macros::*;

use crate::{
    types::{
        Asset, AssetChunk, AssetID, AssetProperties, AssetQuery, Blob, ChunkID, CreateStrategyArgs,
        HeaderField, HttpRequest, HttpResponse, StreamingStrategy, StreamingCallbackToken, StreamingCallbackHttpResponse,
    },
    utils::{generate_url, get_asset_id},
};

pub struct ChunkState {
    pub in_production: bool,
    pub chunk_count: u32,
    pub chunks: HashMap<ChunkID, AssetChunk>,
    pub asset_count: u128,
    pub assets: HashMap<AssetID, Asset>,
}

impl Default for ChunkState {
    fn default() -> Self {
        Self {
            in_production: false,
            chunk_count: 0,
            chunks: HashMap::new(),
            asset_count: 0,
            assets: HashMap::new(),
        }
    }
}

impl ChunkState {
    fn increment(&mut self) -> u32 {
        self.chunk_count += 1;
        self.chunk_count
    }

    fn get_asset_id(&mut self) -> String {
        let id = self.asset_count;
        self.asset_count += 1;
        format!("{}", id)
    }
}

thread_local! {
    pub static CHUNK_STATE: RefCell<ChunkState> = RefCell::default();
}

#[update]
#[candid_method(update)]
pub fn create_chunk(batch_id: String, content: Blob, order:u32) -> u32 {
    CHUNK_STATE.with(|state| {
        let mut state = state.borrow_mut();
        let id = state.increment();
        let checksum: u32 = crc32fast::hash(&content);

        let asset_chunk = AssetChunk {
            batch_id,
            checksum,
            content,
            created_at: ic_cdk::api::time(),
            file_name: "".to_string(),
            id,
            order,
            owner: ic_cdk::caller(),
        };

        state.chunks.insert(id, asset_chunk);
        id
    })
}

fn validate_chunks(chunk_ids: &[u32], batch_id: &str, caller: Principal, state: &mut ChunkState) -> Result<Vec<AssetChunk>, String> {
    let mut chunks_to_commit = Vec::new();
    for &chunk_id in chunk_ids {
        match state.chunks.get(&chunk_id) {
            None => return Err("Invalid Chunk Id".to_string()),
            Some(chunk) if chunk.batch_id == batch_id && chunk.owner == caller => chunks_to_commit.push(chunk.clone()),
            _ => return Err("Not Owner of Chunk".to_string()),
        }
    }
    Ok(chunks_to_commit)
}


#[update]
#[candid_method(update)]
pub fn commit_batch(
    batch_id: String,
    chunk_ids: Vec<u32>,
    asset_properties: AssetProperties,
) -> Result<AssetID, String> {
    ic_cdk::println!("{:?}", chunk_ids);
    let caller = ic_cdk::caller();
    CHUNK_STATE.with(|state| {
        let mut state = state.borrow_mut();
        let asset_id = state.get_asset_id();
        let canister_id = ic_cdk::id();
        
        // NOTES: remove chunk_ids, although it is more direct in reading 
        // it also adds something else the UI will need to keep track of
        let mut chunks_to_commit = validate_chunks(&chunk_ids, &batch_id, caller, &mut state)?;

        chunks_to_commit.sort_by_key(|chunk| chunk.order);
        ic_cdk::println!("{:?}", chunks_to_commit.iter().map(|chunk| chunk.order).collect::<Vec<_>>());

        let mut asset_content = Vec::new();
        let mut content_size = 0;
        let mut asset_checksum = 0;
        let modulo_value = 400_000_000;
        
        for chunk in chunks_to_commit.iter() {
            asset_content.push(chunk.content.clone());
            asset_checksum = (asset_checksum + chunk.checksum) % modulo_value;
            content_size += chunk.content.len();
        }
        
        if asset_checksum != asset_properties.checksum {
            return Err("Invalid Checksum: Chunk Missing".to_string());
        }

        for chunk in chunks_to_commit.iter() {
            state.chunks.remove(&chunk.id);
        }

        let asset = Asset {
            canister_id,
            chunk_size: asset_content.len() as u32,
            content: Some(asset_content),
            content_encoding: asset_properties.content_encoding,
            content_type: asset_properties.content_type,
            filename: asset_properties.filename,
            content_size: content_size as u32,
            created_at: ic_cdk::api::time(),
            id: asset_id.clone(),
            owner: caller.to_string(),
            url: generate_url(asset_id.clone(), state.in_production),
        };
        
        state.assets.insert(asset_id.clone(), asset);
        Ok(asset_id)
    })
}

#[query]
#[candid_method(query)]
pub fn get(asset_id: String) -> Result<AssetQuery, String> {
    CHUNK_STATE.with(|state| {
        let state = state.borrow();
        match state.assets.get(&asset_id) {
            None => Err("Asset Not Found".to_string()),
            Some(asset) => Ok(AssetQuery::from(asset)),
        }
    })
}

#[query]
#[candid_method(query)]
pub fn version() -> Nat {
    Nat::from(1)
}

// http working

#[query]
#[candid_method(query)]
pub fn http_request(request: HttpRequest) -> HttpResponse {
    let not_found = b"Asset Not Found".to_vec();
    let asset_id = get_asset_id(request.url);
    CHUNK_STATE.with(|state| {
        let state = state.borrow();
        match state.assets.get(&asset_id) {
            None => HttpResponse {
                body: not_found,
                status_code: 404,
                headers: vec![],
                streaming_strategy: None,
            },
            Some(asset) => {
                let filename = format!("attachment; filename={}", asset.filename);
                HttpResponse {
                    body: asset.content.clone().unwrap()[0].clone(),
                    status_code: 200,
                    headers: vec![
                        HeaderField("Content-Type".to_string(), asset.content_type.clone()),
                        HeaderField("accept-ranges".to_string(), "bytes".to_string()),
                        HeaderField("Content-Disposition".to_string(), filename),
                        HeaderField(
                            "cache-control".to_string(),
                            "private, max-age=0".to_string(),
                        ),
                    ],
                    streaming_strategy: create_strategy(CreateStrategyArgs {
                        asset_id: asset_id.clone(),
                        chunk_index: 0,
                        data_chunks_size: Nat::from(asset.chunk_size),
                    }),
                }
            }
        }
    })
}

fn create_strategy(arg: CreateStrategyArgs) -> Option<StreamingStrategy> {
    match create_token(arg) {
        None => None,
        Some(token) => {
            let id = ic_cdk::id();
            Some(StreamingStrategy::Callback {
                token,
                callback: Func{ principal: id, method: "http_request_streaming_callback".to_string(),},
            })
        }
    }
}

fn create_token(arg: CreateStrategyArgs) -> Option<StreamingCallbackToken> {
    let v = arg.chunk_index + 1;
    if v >= arg.data_chunks_size {
        return None
    }
    Some(StreamingCallbackToken {
        asset_id: arg.asset_id,
        chunk_index: v,
        content_encoding: "gzip".to_string()
    })
}

#[query]
#[candid_method(query)]
pub fn http_request_streaming_callback(token_arg: StreamingCallbackToken) -> StreamingCallbackHttpResponse {
    CHUNK_STATE.with(|state| {
        let state = state.borrow();

        if let Some(asset) = state.assets.get(&token_arg.asset_id) {
            let arg = CreateStrategyArgs {
                asset_id: token_arg.asset_id,
                chunk_index: token_arg.chunk_index,
                data_chunks_size: Nat::from(asset.chunk_size),
            };
            let token = create_token(arg);

            StreamingCallbackHttpResponse {
                token,
                body: asset.content.as_ref().map(|content| content[token_arg.chunk_index as usize].clone()),
            }
        } else {
            panic!("Asset ID not found");
        }
    })
}