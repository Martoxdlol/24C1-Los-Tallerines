pub fn js_api_stream_create(name: &str) -> String {
    format!("$JS.API.STREAM.CREATE.{}", name)
}

pub fn js_api_consumer_create(name: &str) -> String {
    format!("$JS.API.STREAM.CREATE.{}", name)
}

pub fn js_api_consumer_next(stream_name: &str, consumer_name: &str) -> String {
    format!(
        "$JS.API.CONSUMER.MSG.NEXT.{}.{}",
        stream_name, consumer_name
    )
}
