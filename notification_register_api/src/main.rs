use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use aws_sdk_dynamodb::types::AttributeValue;

const SNS_ARN: &str = "arn:aws:sns:eu-central-1:284535252702:app/GCM/sensor_notification";
const SNS_ARN_TABLE: &str = "notification_devices"; // Collection of all notification registred devices

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

    let user_id = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("uid"))
        .unwrap();

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sns::Client::new(&config);
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);

    let result = client.create_platform_endpoint()
    .platform_application_arn(SNS_ARN)
    .token(device_token)
    .send()
    .await;

    match result {
        Ok(result) => {
          let res:Response<Body>  =  Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body("OK".into())
            .map_err(Box::new)?;

            dynamodb_client.put_item()
                .table_name(SNS_ARN_TABLE)
                .item("user_id", AttributeValue::S(user_id.to_string()))
                .item("arn", AttributeValue::S(result.endpoint_arn.unwrap()))
                .send().await.unwrap();

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
