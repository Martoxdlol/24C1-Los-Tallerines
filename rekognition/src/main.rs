use aws_sdk_rekognition::{self as rekognition, primitives::Blob, types::Image};
use dotenv::dotenv;
use futures::executor;

#[::tokio::main]
pub async fn main() {}

pub fn reconocer_imagen(ruta: &str) -> Result<(), String> {
    dotenv().ok();
    let input = std::fs::read(ruta).map_err(|e| e.to_string())?;

    let arn_from_env =
        std::env::var("REKOGNITION_PROJECT_VERSION_ARN").map_err(|e| e.to_string())?;

    executor::block_on(reconocer_async(&arn_from_env, input)).map_err(|e| e.to_string())
}

async fn reconocer_async(arn: &str, bytes: Vec<u8>) -> Result<(), rekognition::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_rekognition::Client::new(&config);

    // load file bytes

    let result = client
        .detect_custom_labels()
        .project_version_arn(arn)
        .image(Image::builder().bytes(Blob::new(bytes)).build())
        .send()
        .await?;

    println!("{:#?}", result.custom_labels.unwrap_or_default());

    Ok(())
}
