pub enum Message {
    // 'topico', 'payload', 'replyTo'
    Pub(String, Vec<u8>, Option<String>),
    // 'topico', 'id
    Sub(String, String),
}
