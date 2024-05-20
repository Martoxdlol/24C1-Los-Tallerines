#[derive(Debug)]
pub enum DeserializationError {
    InvalidData,
    MissingField,
    ExtraField,
}
