use lib::jet_stream::{consumer_info::ConsumerInfo, stream_info::StreamInfo};

#[derive(Debug)]
pub enum ActualizacionJS {
    Stream(StreamInfo),
    Consumer(ConsumerInfo),
    StreamEliminado(String),
    ConsumerEliminado(String),
}
