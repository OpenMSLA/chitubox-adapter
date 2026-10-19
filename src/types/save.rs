use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Debug)]
pub struct SaveFileOutgoing {
    #[serde(rename = "MsgType")]
    pub msg_type: String,

    #[serde(rename = "FilePath")]
    pub file_path: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct SaveFileIncoming {
    #[serde(rename = "MsgType")]
    pub msg_type: String,

    #[serde(rename = "Data")]
    pub data: SaveData,
}

#[derive(Deserialize, Debug)]
pub struct SaveData {
    #[serde(rename = "SavePath")]
    pub save_path: PathBuf,

    #[serde(rename = "VerifyFlag")]
    pub verify_flag: bool,
}
