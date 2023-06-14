pub fn generate_url(asset_id: String, in_production: bool) -> String{
    let canister_id = ic_cdk::id();
    match in_production{
        true => format!("https://{canister_id}.raw.ic0.app/asset/{asset_id}"),
        false => format!("http://{canister_id}.localhost:8080/asset/{asset_id}")
    }
}

pub fn get_asset_id(url: String) -> String{
    let url_split_by_path = url.split('/').collect::<Vec<&str>>();
    let last_elem = url_split_by_path[url_split_by_path.len() - 1];
    let first_elem: Vec<&str> = last_elem.split('?').collect();
    first_elem[0].to_string()
}