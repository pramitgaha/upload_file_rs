use std::cell::RefCell;

use candid::{candid_method, Principal, CandidType, Nat, Deserialize};
use ic_cdk::api::management_canister::main::{
    CreateCanisterArgument, CanisterIdRecord, CanisterSettings, CanisterInstallMode, InstallCodeArgument,
};

pub async fn get_an_address(caller: &Principal) -> Principal{
    ic_cdk::println!("{}", caller.clone());
    let canister_setting = CanisterSettings{
        controllers: Some(vec![caller.clone(), ic_cdk::id()]),
        compute_allocation: Some(Nat::from(0_u64)),
        memory_allocation: Some(Nat::from(0_u64)),
        freezing_threshold: Some(Nat::from(0_u64)),
    };
    let args = CreateCanisterArgument{
        settings: Some(canister_setting)
    };
    let (canister_id,): (CanisterIdRecord,) = match ic_cdk::api::call::call_with_payment(Principal::management_canister(), "create_canister", (args,), 20_00_000_000_000).await
    {
        Ok(x) => x,
        Err((_, _)) => (CanisterIdRecord{
            canister_id: candid::Principal::anonymous()
        },),
    };
    canister_id.canister_id
}

pub async fn install_wasm(wasm: Vec<u8>, canister_id: Principal, args: Vec<u8>,) -> bool{
    let install_config = InstallCodeArgument{
        mode: CanisterInstallMode::Install,
        wasm_module: wasm,
        canister_id,
        arg: args
    };
    match ic_cdk::api::call::call(Principal::management_canister(), "install_code", (install_config,)).await
    {
        Ok(x) => x,
        Err((rejection_code, msg)) =>{
            ic_cdk::println!("{:?} {:?}", rejection_code, msg);
            return false
        }
    }
    true
}