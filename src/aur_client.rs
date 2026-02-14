use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurPackage {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Maintainer")]
    pub maintainer: Option<String>,
    #[serde(rename = "NumVotes")]
    pub votes: Option<i32>,
    #[serde(rename = "Popularity")]
    pub popularity: Option<f64>,
    #[serde(rename = "OutOfDate")]
    pub out_of_date: Option<i64>,
    #[serde(rename = "LastModified")]
    pub last_modified: Option<i64>,
    #[serde(rename = "URL")]
    pub url: Option<String>,
    #[serde(rename = "URLPath")]
    pub url_path: Option<String>,
    #[serde(rename = "Depends")]
    pub depends: Option<Vec<String>>,
    #[serde(rename = "MakeDepends")]
    pub makedepends: Option<Vec<String>>,
    #[serde(rename = "OptDepends")]
    pub optdepends: Option<Vec<String>>,
    #[serde(rename = "Conflicts")]
    pub conflicts: Option<Vec<String>>,
    #[serde(rename = "License")]
    pub license: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct AurResponse {
    results: Vec<AurPackage>,
}

pub async fn search_aur(query: &str) -> Result<Vec<AurPackage>, Box<dyn std::error::Error>> {
    let encoded_query = urlencoding::encode(query);
    let url = format!(
        "https://aur.archlinux.org/rpc?v=5&type=search&arg={}",
        encoded_query
    );

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    let aur_response: AurResponse = response.json().await?;

    Ok(aur_response.results)
}

pub async fn get_package_info(package_name: &str) -> Result<AurPackage, Box<dyn std::error::Error>> {
    let encoded_query = urlencoding::encode(package_name);
    let url = format!(
        "https://aur.archlinux.org/rpc?v=5&type=info&arg={}",
        encoded_query
    );

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    let aur_response: AurResponse = response.json().await?;

    aur_response.results.into_iter().next()
        .ok_or_else(|| "Package not found".into())
}

pub async fn get_recent_packages(count: usize) -> Result<Vec<AurPackage>, Box<dyn std::error::Error>> {
    // Test connection with a simple search
    let url = "https://aur.archlinux.org/rpc?v=5&type=search&by=name&arg=";
    
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let mut aur_response: AurResponse = response.json().await?;
    
    // Sort by last modified
    aur_response.results.sort_by(|a, b| {
        b.last_modified.unwrap_or(0).cmp(&a.last_modified.unwrap_or(0))
    });
    
    aur_response.results.truncate(count);
    Ok(aur_response.results)
}
