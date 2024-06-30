use std::path::Path;

use aws_sdk_rekognition::{
    self as rekognition,
    primitives::Blob,
    types::{CustomLabel, Image},
};
use dotenv::dotenv;
use tokio::runtime::Runtime;

pub fn reconocer_imagen<P: AsRef<Path>>(ruta: P) -> Result<Vec<CustomLabel>, String> {
    dotenv().ok();
    let input = std::fs::read(ruta).map_err(|e| e.to_string())?;

    let arn_from_env = std::env::var("AWS_PROJECT_ARN").map_err(|e| e.to_string())?;

    let runtime = Runtime::new().map_err(|e| e.to_string())?;

    let resultado = runtime
        .block_on(reconocer_async(&arn_from_env, input))
        .map_err(|e| {
            format!(
                "No se pudo utilizar el modelo para reconocer la imagen. {}",
                e
            )
        })?;

    Ok(resultado)
}

async fn reconocer_async(
    arn: &str,
    bytes: Vec<u8>,
) -> Result<Vec<CustomLabel>, rekognition::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_rekognition::Client::new(&config);

    // load file bytes

    let result = client
        .detect_custom_labels()
        .project_version_arn(arn)
        .image(Image::builder().bytes(Blob::new(bytes)).build())
        .send()
        .await?;

    Ok(result.custom_labels.unwrap_or_default())
}
