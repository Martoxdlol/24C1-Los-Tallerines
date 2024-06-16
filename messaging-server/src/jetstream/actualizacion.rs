use lib::jet_stream::stream_info::StreamInfo;

#[derive(Debug)]
pub enum ActualizacionJS {
    Stream(StreamInfo),
    StreamEliminado(String),
}
