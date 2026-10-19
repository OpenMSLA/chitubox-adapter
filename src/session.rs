use anyhow::{Result, bail};
use regex::Regex;
use serde_json::Value;
use tokio::{
    fs,
    io::{self, AsyncWriteExt},
    net::TcpStream,
};
use tracing::{debug, error, info};

use crate::{
    streaming_json_reader::StreamingJsonReader,
    types::{
        load_window::{LoadWindowIncoming, LoadWindowOutgoing},
        save::{SaveFileIncoming, SaveFileOutgoing},
    },
};

pub struct Session {
    pub stream: TcpStream,

    json_reader: StreamingJsonReader,
}

impl Session {
    /// Creates new session
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            json_reader: StreamingJsonReader::default(),
        }
    }

    /// Runs session working loop
    pub async fn run(&mut self) -> Result<()> {
        loop {
            self.stream.readable().await?;
            let mut temp_buf = vec![0; 1024];

            match self.stream.try_read(&mut temp_buf) {
                Ok(0) => {
                    debug!("No more data to read. Exiting...");
                    break; // if client disconnected
                },
                Ok(n) => {
                    let values = self.json_reader.on_frame(&temp_buf[..n])?;
                    for value in values {
                        self.handle_packet(value).await?;
                    }
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                },
                Err(e) => {
                    error!("Read error: {}", e)
                },
            }
        }

        Ok(())
    }

    /// Handle next tcp json value
    async fn handle_packet(&mut self, json: Value) -> Result<()> {
        match json.get("MsgType") {
            Some(t) => match t.as_str() {
                Some("LoadWindow") => self.handle_load_window(json).await?,
                Some("SaveFile") => self.handle_save_request(json).await?,

                Some(t) => info!("Ignored packet {}", t),
                None => bail!("MsgType is not a string!"),
            },
            None => bail!("Cannot determine message type: MsgType not present"),
        }
        Ok(())
    }

    /// Handle `LoadWindow` packet
    async fn handle_load_window(&mut self, json: Value) -> Result<()> {
        let load_win: LoadWindowIncoming = serde_json::from_value(json)?;

        println!("Supported extensions: ");
        for ext in Self::extract_supported_extensions(load_win.data.printer_info.slice_format)? {
            println!("{}", ext);
        }

        self.send_load_window_ack(load_win.win_type).await?;

        self.send_save_request().await?;

        self.close_session().await?;
        Ok(())
    }

    /// Send back `LoadWindow` ACK
    async fn send_load_window_ack(&mut self, win_type: u8) -> Result<()> {
        let payload = LoadWindowOutgoing {
            msg_type: "LoadWindow".to_string(),
            result: true,
            handle: if win_type == 1 {
                "network_send".to_string()
            } else {
                "network_manager".to_string()
            },
            win_type,
        };

        self.stream.writable().await?;
        self.stream
            .write_all(&serde_json::to_string(&payload)?.into_bytes())
            .await?;

        debug!("LoadWindow ACK sent!");
        Ok(())
    }

    /// Send request to local saving
    async fn send_save_request(&mut self) -> Result<()> {
        let save_path = std::env::current_dir()?.join("saves");

        if !save_path.exists() {
            fs::create_dir_all(&save_path).await?;
        }

        let payload = SaveFileOutgoing {
            msg_type: "SaveFile".to_string(),
            file_path: save_path.join("model.zip"),
        };

        self.stream.writable().await?;
        self.stream
            .write_all(&serde_json::to_string(&payload)?.into_bytes())
            .await?;

        debug!("SaveRequest sent!");
        Ok(())
    }

    /// Handle local saving response
    async fn handle_save_request(&mut self, json: Value) -> Result<()> {
        let saved_file: SaveFileIncoming = serde_json::from_value(json)?;

        info!("File saved: {:?}!", saved_file.data.save_path);
        Ok(())
    }

    /// Extract supported extension vector from `CHITUBOX slicer(*.cbddlp)`-like strings
    fn extract_supported_extensions(ext: Vec<String>) -> Result<Vec<String>> {
        let re = Regex::new(r"\*(\.[a-zA-Z0-9]+)")?;

        Ok(ext
            .iter()
            .filter_map(|e| re.captures(e).map(|ext| ext[1].to_string()))
            .collect())
    }

    /// Closes TCP session
    async fn close_session(&mut self) -> Result<()> {
        self.stream.shutdown().await?;

        Ok(())
    }
}
