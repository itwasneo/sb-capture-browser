use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Bookmarks {
    pub checksum: String,
    pub roots: Roots,
    pub sync_metadata: String,
    pub version: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Roots {
    pub bookmark_bar: Folder,
    pub other: Folder,
    pub synced: Folder,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Folder {
    pub children: Vec<Child>,
    pub date_added: String,
    pub date_last_used: String,
    pub date_modified: Option<String>,
    pub guid: String,
    pub id: String,
    pub meta_info: Option<MetaInfo>,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetaInfo {
    #[serde(rename = "power_bookmark_meta")]
    pub power_bookmark_meta: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Child {
    Folder(Folder),
    Bookmark {
        date_added: String,
        date_last_used: String,
        date_modified: Option<String>,
        guid: String,
        id: String,
        name: String,
        meta_info: Option<MetaInfo>,
        #[serde(rename = "type")]
        type_: String,
        url: String,
    },
}
