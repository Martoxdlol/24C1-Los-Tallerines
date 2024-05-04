pub enum ClientMessages<'a> {
    Connect(String),
    Pub {
        topic: &'a str,
        len_message: Option<usize>,
        message: &'a str,
    },
    Hpub {
        topic: &'a str,
    },
    Sub {
        topic: &'a str,
        subscription_id: Option<u8>,
    },
    Unsub {
        subscription_id: Option<u8>,
    },
    Err(String)
}