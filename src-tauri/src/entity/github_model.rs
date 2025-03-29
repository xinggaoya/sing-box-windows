use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    pub url: String,
    pub assets_url: String,
    pub upload_url: String,
    pub html_url: String,
    pub id: i64,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: Option<String>,
    pub assets: Vec<Asset>,
    pub tarball_url: Option<String>,
    pub zipball_url: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub url: String,
    pub id: i64,
    pub node_id: String,
    pub name: String,
    pub label: Option<String>,
    pub content_type: String,
    pub state: String,
    pub size: i64,
    pub download_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub browser_download_url: String,
}
