use ic_stable_structures::memory_manager::{MemoryId, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use crate::file::MEMORY_MANAGER;
use crate::types::{AssetChunk, Asset};

// A memory for upgrades, where data from the heap can be serialized/deserialized.
const UPGRADES: MemoryId = MemoryId::new(0);

// A memory for the StableBTreeMap we're using. A new memory should be created for
// every additional stable structure.
const STABLE_CHUNK_BTREE: MemoryId = MemoryId::new(1);
const STABLE_ASSET_BTREE: MemoryId = MemoryId::new(2);

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

pub fn init_chunk_stable_data() -> StableBTreeMap<u32, AssetChunk, StableMemory> {
    StableBTreeMap::init(get_chunk_stable_btree_memory())
}

pub fn init_asset_stable_data() -> StableBTreeMap<u128, Asset, StableMemory>{
    StableBTreeMap::init(get_asset_stable_btree_memory())
}