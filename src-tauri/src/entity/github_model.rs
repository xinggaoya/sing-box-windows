use serde::{Deserialize};

#[derive(Deserialize)]
#[allow(dead_code)]
struct User {
    login: String,
    id: u64,
    node_id: String,
    avatar_url: String,
    gravatar_id: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    #[serde(rename = "type")]
    user_type: String,
    user_view_type: String,
    site_admin: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AssetUploader {
    login: String,
    id: u64,
    node_id: String,
    avatar_url: String,
    gravatar_id: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    #[serde(rename = "type")]
    user_type: String,
    user_view_type: String,
    site_admin: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Asset {
    pub url: String,
    id: u64,
    node_id: String,
    pub name: String,
    pub label: String,
    uploader: AssetUploader,
    content_type: String,
    state: String,
    size: u64,
    download_count: u64,
    created_at: String,
    updated_at: String,
    pub browser_download_url: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Reactions {
    url: String,
    total_count: i32,
    #[serde(rename = "+1")]
    plus_one: i32,
    #[serde(rename = "-1")]
    minus_one: i32,
    laugh: i32,
    hooray: i32,
    confused: i32,
    heart: i32,
    rocket: i32,
    eyes: i32,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Release {
    url: String,
    assets_url: String,
    upload_url: String,
    html_url: String,
    id: u64,
    author: User,
    node_id: String,
    tag_name: String,
    target_commitish: String,
    name: String,
    draft: bool,
    prerelease: bool,
    created_at: String,
    published_at: String,
    pub assets: Vec<Asset>,
    tarball_url: String,
    zipball_url: String,
    body: String,
    reactions: Reactions,
}