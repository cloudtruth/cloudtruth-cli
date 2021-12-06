const LATEST_CHECK_URL: &str =
    "https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/latest";
const BINARY_VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_latest_version_from_server() -> String {
    let client = reqwest::Client::builder().build().unwrap();
    let request = client
        .request(reqwest::Method::GET, LATEST_CHECK_URL)
        .build()
        .unwrap();
    let mut response = client.execute(request).unwrap();
    let status = response.status();
    let content = response.text().unwrap();

    if !status.is_client_error() && !status.is_server_error() {
        let value: serde_json::Value = serde_json::from_str(&content).unwrap();
        if let Some(dict) = value.as_object() {
            if let Some(tag_value) = dict.get("tag_name") {
                if let Some(tag_str) = tag_value.as_str() {
                    return tag_str.to_string();
                }
            }
        }
    }

    "0.0.0".to_string()
}

pub fn get_latest_version() -> String {
    get_latest_version_from_server()
}

pub fn binary_version() -> String {
    BINARY_VERSION.to_string()
}
