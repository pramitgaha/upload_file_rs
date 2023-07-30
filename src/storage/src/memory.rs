use std::cell::RefCell;

use crate::types::{Asset, Chunk, State};
use candid::{candid_method, Nat};
use ic_cdk_macros::{init, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Memory};

// A memory for upgrades, where data from the heap can be serialized/deserialized.
const UPGRADES: MemoryId = MemoryId::new(0);

// A memory for the StableBTreeMap we're using. A new memory should be created for
// every additional stable structure.
const STABLE_CHUNK_BTREE: MemoryId = MemoryId::new(1);
const STABLE_ASSET_BTREE: MemoryId = MemoryId::new(2);

#[update]
#[candid_method(update)]
pub fn chunk_memory_size(){
    let s = MEMORY_MANAGER.with(|m| m.borrow().get(STABLE_CHUNK_BTREE));
    // s.grow(100000);
    let size = s.size();
    ic_cdk::println!("{}", size);
}

#[update]
#[candid_method(update)]
pub fn asset_memory_size(){
    let s = MEMORY_MANAGER.with(|m| m.borrow().get(STABLE_ASSET_BTREE));
    // s.grow(100000);
    let size = s.size();
    ic_cdk::println!("{}", size);
}

pub type StableMemory = VirtualMemory<DefaultMemoryImpl>;

pub fn get_upgrades_memory() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(UPGRADES))
}

pub fn get_chunk_stable_btree_memory() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(STABLE_CHUNK_BTREE))
}

pub fn get_asset_stable_btree_memory() -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(STABLE_ASSET_BTREE))
}

pub fn init_chunk_stable_data() -> StableBTreeMap<u128, Chunk, StableMemory> {
    StableBTreeMap::init(get_chunk_stable_btree_memory())
}

pub fn init_asset_stable_data() -> StableBTreeMap<u128, Asset, StableMemory> {
    StableBTreeMap::init(get_asset_stable_btree_memory())
}

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    pub static STATE: RefCell<State> = RefCell::default();
}

#[init]
#[candid_method(init)]
pub fn init() {}

#[update]
#[candid_method(update)]
pub async fn is_full() -> bool{
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