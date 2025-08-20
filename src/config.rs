use std::{error::Error, fs::File, io::{BufReader, BufWriter}};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct Config {
    pub bind :String,
    pub timeout :i64,
    pub server :String,
    pub waiting_minecraft_packet :u64,
    pub packet_per_sec :u64,
    pub proxy_protocol_v2 :bool
}

impl Config {
    pub fn load(path :&str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config :Config = serde_yaml::from_reader(reader)?;
        Ok(config)
    }

    pub fn save(path :&str) -> Result<(), Box<dyn Error>> {
        let def = Self::default();
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, &def)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            bind: "0.0.0.0:25565".to_string(),
            timeout: 10,
            server: "127.0.0.1:25566".to_string(),
            waiting_minecraft_packet: 10,
            packet_per_sec: 500,
            proxy_protocol_v2: false
        }
    }
}