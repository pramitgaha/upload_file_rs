use std::{cell::RefCell, collections::HashMap};
use candid::{candid_method, Nat, Func};
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};
use ic_stable_structures::{StableBTreeMap, memory_manager::MemoryManager, DefaultMemoryImpl, writer::Writer, Memory};
use crate::{
    types::{
        Asset, AssetChunk, AssetProperties, AssetQuery, ChunkID, CreateStrategyArgs,
        HeaderField, HttpRequest, HttpResponse, StreamingStrategy, StreamingCallbackToken, StreamingCallbackHttpResponse, Blob, Content,
    },
    utils::{generate_url, get_asset_id}, memory::{StableMemory, init_chunk_stable_data, init_asset_stable_data, init_content_stable_data},
};

#[derive(Deserialize, Serialize)]
pub struct ChunkState {
    pub in_production: bool,
    pub chunk_count: u32,
    #[serde(skip, default = "init_chunk_stable_data")]
    pub chunks: StableBTreeMap<ChunkID, AssetChunk, StableMemory>,
    pub asset_count: u128,
    #[serde(skip, default = "init_asset_stable_data")]
    pub assets: StableBTreeMap<u128, Asset, StableMemory>,
}

impl Default for ChunkState {
    fn default() -> Self {
        Self {
            in_production: false,
            chunk_count: 0,
            chunks: init_chunk_stable_data(),
            asset_count: 0,
            assets: init_asset_stable_data(),
        }
    }
}

impl ChunkState {
    fn increment(&mut self) -> u32 {
        self.chunk_count += 1;
        self.chunk_count
    }

    fn get_asset_id(&mut self) -> u128 {
        let id = self.asset_count;
        self.asset_count += 1;
        id
    }
}

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    pub static STATE: RefCell<ChunkState> = RefCell::default();
}

#[update]
#[candid_method(update)]
pub fn create_chunk(content: Vec<u8>, order:u32) -> u32 {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let id = state.increment();
        let checksum: u32 = crc32fast::hash(&content);
        let content = content.iter().map(|b| b.clone()).collect();
        let asset_chunk = AssetChunk {
            checksum,
            content,
            created_at: ic_cdk::api::time(),
            id: id.clone(),
            order,
            owner: ic_cdk::caller(),
        };
        state.chunks.insert(id, asset_chunk);
        id
    })
}

#[update]
#[candid_method(update)]
pub fn commit_batch(
    chunk_ids: Vec<u32>,
    asset_properties: AssetProperties,
) -> Result<u128, String> {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let caller = ic_cdk::caller();

        // Collect and verify chunks
        let mut chunks_to_commit = Vec::new();
        for chunk_id in chunk_ids.iter() {
            match state.chunks.get(chunk_id) {
                None => return Err("Invalid chunk ID.".to_string()),
                Some(chunk) if chunk.owner != caller => return Err("Caller does not own the chunk.".to_string()),
                Some(chunk) => chunks_to_commit.push((chunk.id, chunk.order)),
            }
        }

        // Sort chunks by order
        chunks_to_commit.sort_by_key(|chunk| chunk.1);

        // Accumulate content and compute checksum
        let modulo_value = 400_000_000;
        let mut asset_content = init_content_stable_data();
        let mut asset_checksum = 0;
        let mut content_size = 0;
        for (chunk_id, _) in chunks_to_commit.iter() {
            let chunk = state.chunks.get(chunk_id).unwrap();
            ic_cdk::println!("{}", chunk.order);
            asset_content.insert(chunk.order, Content(chunk.content.clone()));
            asset_checksum = (asset_checksum + chunk.checksum) % modulo_value;
            content_size += chunk.content.len();
        }

        // Verify checksum
        if asset_checksum != asset_properties.checksum {
            return Err("Checksum mismatch.".to_string());
        }

        // Remove committed chunks
        for (chunk_id, _) in chunks_to_commit.iter() {
            state.chunks.remove(chunk_id);
        }

        // Create and insert new asset
        let asset_id = state.get_asset_id();
        let canister_id = ic_cdk::id();
        let asset = Asset {
            canister_id,
            chunk_size: asset_content.len() as u32,
            content: asset_content,
            content_encoding: asset_properties.content_encoding,
            content_type: asset_properties.content_type,
            filename: asset_properties.filename,
            content_size: content_size as u32,
            created_at: ic_cdk::api::time(),
            id: asset_id,
            owner: caller,
            url: generate_url(asset_id, state.in_production),
        };
        state.assets.insert(asset_id, asset);

        Ok(asset_id)
    })
}

#[query]
#[candid_method(query)]
pub fn assets_list() -> HashMap<u128, AssetQuery>{
    STATE.with(|state|{
        let state = state.borrow();
        state.assets.iter().map(|(key, asset)| (key.clone(), AssetQuery::from(&asset))).collect()
    })
}

#[query]
#[candid_method(query)]
pub fn chunk_availabity_check(chunk_id: u32) -> bool{
    STATE.with(|state|{
        let state = state.borrow();
        match state.chunks.get(&chunk_id){
            None => false,
            Some(_chunk) => true
        }
    })
}

#[update]
#[candid_method(update)]
pub fn delete_asset(asset_id: u128) -> Result<String, String>{
    let caller = ic_cdk::caller();
    STATE.with(|state|{
        let mut state = state.borrow_mut();
        match state.assets.get(&asset_id){
            None => return Err("Invalid asset id".to_string()),
            Some(asset) => {
                if asset.owner != caller{
                    return Err("Unauthorized".to_string())
                }
                state.assets.remove(&asset_id);
                Ok("Success".to_string())
            }
        }
    })
}

#[query]
#[candid_method(query)]
pub fn get(asset_id: u128) -> Result<AssetQuery, String> {
    STATE.with(|state| {
        let state = state.borrow();
        match state.assets.get(&asset_id) {
            None => Err("Asset Not Found".to_string()),
            Some(ref asset) => {
                Ok(AssetQuery::from(asset))
            },
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
// #[candid_method(query)]
pub fn http_request(request: HttpRequest) -> HttpResponse {
    let not_found = b"Asset Not Found".iter().map(|b| b.clone()).collect();
    let asset_id = get_asset_id(request.url);
    STATE.with(|state| {
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
                    body: asset.content.get(&0).unwrap().0.clone(),
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

fn create_token(arg: CreateStrategyArgs) -> Option<StreamingCallbackToken>{
    let v = arg.chunk_index.clone() + 1;
    if v >= arg.data_chunks_size{
        return None
    }
    Some(StreamingCallbackToken{
        asset_id: arg.asset_id,
        chunk_index: arg.chunk_index.clone() + 1,
        content_encoding: "gzip".to_string()
    })
}

#[query]
#[candid_method(query)]
pub fn http_request_streaming_callback(token_arg: StreamingCallbackToken) -> StreamingCallbackHttpResponse{
    STATE.with(|state|{
        let state = state.borrow();
        match state.assets.get(&token_arg.asset_id){
            None => panic!("asset id not found"),
            Some(asset) => {
                let arg = CreateStrategyArgs{
                    asset_id: token_arg.asset_id.clone(),
                    chunk_index: token_arg.chunk_index,
                    data_chunks_size: Nat::from(asset.chunk_size.clone()),
                };
                let token = create_token(arg);
                StreamingCallbackHttpResponse{
                    token,
                    body: asset.content.get(&token_arg.chunk_index).unwrap().0.clone()
                }
            }
        }
    })
}

// canister management
#[pre_upgrade]
pub fn pre_upgrade(){
    let mut state_bytes = vec![];
    STATE
        .with(|s| ciborium::ser::into_writer(&*s.borrow(), &mut state_bytes))
        .expect("failed to encode state");
    // Write the length of the serialized bytes to memory, followed by the
    // by the bytes themselves.
    let len = state_bytes.len() as u32;
    let mut memory = crate::memory::get_upgrades_memory();
    let mut writer = Writer::new(&mut memory, 0);
    writer.write(&len.to_le_bytes()).expect("failed to write length");
    writer.write(&state_bytes).expect("failed to write bytes");
}


#[post_upgrade]
pub fn post_upgrade(){
    let memory = crate::memory::get_upgrades_memory();
    // Read the length of the state bytes.
    let mut state_len_bytes = [0; 4];
    memory.read(0, &mut state_len_bytes);
    let state_len = u32::from_le_bytes(state_len_bytes) as usize;
    // Read the bytes
    let mut state_bytes = vec![0; state_len];
    memory.read(4, &mut state_bytes);
    // Deserialize and set the state.
    let state: ChunkState =
        ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
    STATE.with(|s| *s.borrow_mut() = state);
}

#[update]
#[candid_method(update)]
pub async fn is_full() -> bool{
    // let info: ic_cdk::api::management_canister::main::CanisterInfoResponse = ic_cdk::call(Principal::management_canister(), "", args)
    let arg = ic_cdk::api::management_canister::main::CanisterIdRecord{
        canister_id: ic_cdk::id()
    };
    let (info,) = ic_cdk::api::management_canister::main::canister_status(arg).await.unwrap();
    ic_cdk::println!("{:?}", info);
    let fourty_gb: u64 = 40 * 1024 * 1024 * 1024;
    let max_size = Nat::from(fourty_gb);
    if info.memory_size >= max_size{
        true
    }else{
        false
    }
}