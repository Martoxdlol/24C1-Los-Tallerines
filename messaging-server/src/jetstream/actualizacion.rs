use lib::jet_stream::stream_info::StreamInfo;

pub enum ActualizacionJS {
    Stream(StreamInfo),
    StreamEliminado(String),
}
