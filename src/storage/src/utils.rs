const IN_PROD: bool = false;

pub(crate) fn generate_url(asset_id: u128) -> String{
    let canister_id = ic_cdk::id();
    match IN_PROD{
        true => format!("https://{canister_id}.raw.ic0.app/asset/{asset_id}"),
        false => format!("http://{canister_id}.localhost:8080/asset/{asset_id}")
    }
}