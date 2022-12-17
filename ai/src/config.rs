use std::error::Error;
use std::fs;
use protobuf;
use prost::Message;
use crate::proto;

// Sadly the rust-protobuf crate always generates structs with Option<T>
// fields, even if the field is "required" in proto2. There are accessor
// functions, but they also provide a default value. In general I prefer
// the interface of Prost, so we use rust-protobuf to read the text format
// (since that is unsupported in Prost), and then convert to Prost
pub fn load_config() -> Result<proto::Config, Box<dyn Error>> {
    // Path relative to Cargo.toml
    let config_filepath = "../config/config.pbtxt";
    let file_contents = fs::read_to_string(config_filepath)?;
    let config = protobuf::text_format::parse_from_str(file_contents.as_str())?;
    convert_rust_protobuf_to_prost(config)
}

fn convert_rust_protobuf_to_prost(msg: proto::config::Config) -> Result<proto::Config, Box<dyn Error>> {
    let intermediate_bytes: Vec<u8> = protobuf::Message::write_to_bytes(&msg).unwrap();
    match proto::Config::decode(intermediate_bytes.as_slice()) {
        Ok(m) => Ok(m),
        Err(e) => Err(Box::new(e))
    }
}