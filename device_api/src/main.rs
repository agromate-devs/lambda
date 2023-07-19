use aws_config::load_from_env;
use aws_sdk_dynamodb::{ Client };
use lambda_http::{run, service_fn, Body, Error, Request, Response};
mod router;
use router::router;

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let shared_config = load_from_env().await;
    let client = Client::new(&shared_config);
    Ok(router(event, &client).await.unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
