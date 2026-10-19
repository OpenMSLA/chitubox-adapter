use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct LoadWindowIncoming {
    #[serde(rename = "Data")]
    pub data: Data,

    #[serde(rename = "WinType")]
    pub win_type: u8,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    #[serde(rename = "PrinterInfo")]
    pub printer_info: PrinterInfo,

    #[serde(rename = "WindowProperty")]
    pub window_property: WindowProperty,
}

#[derive(Deserialize, Debug)]
pub struct PrinterInfo {
    #[serde(rename = "PrinterType")]
    pub printer_type: String,

    #[serde(rename = "ResolutionRatioW")]
    pub printer_ratio_w: u64,

    #[serde(rename = "ResolutionRatioH")]
    pub printer_ratio_h: u64,

    #[serde(rename = "SliceFileName")]
    pub slice_file_name: String,

    #[serde(rename = "SliceFormat")]
    pub slice_format: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct WindowProperty {
    #[serde(rename = "Width")]
    pub width: u64,

    #[serde(rename = "Height")]
    pub height: u64,

    #[serde(rename = "Title")]
    pub title: String,

    #[serde(rename = "Url")]
    pub url: String,
}

#[derive(Serialize, Debug)]
pub struct LoadWindowOutgoing {
    #[serde(rename = "MsgType")]
    pub msg_type: String,

    #[serde(rename = "Result")]
    pub result: bool,

    #[serde(rename = "Handle")]
    pub handle: String,

    #[serde(rename = "WinType")]
    pub win_type: u8,
}
