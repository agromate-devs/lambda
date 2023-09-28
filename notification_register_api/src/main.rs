use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};

const SNS_ARN: &str = "arn:aws:sns:eu-central-1:284535252702:app/GCM/sensor_notification";

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Get Device Registration Token from Request
    let device_token = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("token"))
        .unwrap();

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sns::Client::new(&config);
    
    let result = client.create_platform_endpoint()
    .platform_application_arn(SNS_ARN)
    .token(device_token)
    .send()
    .await;

    match result {
        Ok(_) => {
          let res:Response<Body>  =  Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body("OK".into())
            .map_err(Box::new)?;

          Ok(res)
        },
        Err(_) => {
           let res: Response<Body> = Response::builder()
            .status(500)
            .header("content-type", "text/html")
            .body("Error during adding platform endpoint".into())
            .map_err(Box::new)?;
           Ok(res)
        }
    }
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
