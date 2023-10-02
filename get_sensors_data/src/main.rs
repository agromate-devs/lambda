use aws_sdk_dynamodb::{operation::query::QueryOutput, types::AttributeValue, Client};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use std::collections::HashMap;
use helper::get_user_id;

const TABLE_NAME: &str = "sensor_measuration";

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct SensorData {
    uuid: String,      // ESP8266 UUID
    timestamp: String, // Timestamp captured by AWS IoT Core rule
    hour: bool,
    humidity: String,
    media_month: bool,
    soil_humidity: String,
    temperature: String,
}

async fn get_list(client: &Client, uid: &str) -> QueryOutput {
    client
        .query()
        .table_name(TABLE_NAME)
        .index_name("uuid-index") // Use the GSI to query only UUID in table. Since we use a "fake" composite key uuid_timestamp in table
        .key_condition_expression("#uuid_attribute = :uuid")
        .expression_attribute_names("#uuid_attribute", "uuid")
        .expression_attribute_values(":uuid", AttributeValue::S(uid.to_string()))
        .send()
        .await
        .unwrap()
}

fn hashmap_to_lists(measurements: Vec<HashMap<String, AttributeValue>>) -> Vec<SensorData> {
    let mut list_of_measurements: Vec<SensorData> = Vec::new();

    for measurement in measurements {
        list_of_measurements.push(SensorData {  // Wrap dynamoDB data in serializable structure
            uuid: measurement.get("uuid").unwrap().as_s().unwrap().to_string(),
            timestamp: measurement
                .get("timestamp")
                .unwrap()
                .as_n()
                .unwrap()
                .to_string(),
            hour: *measurement.get("hour").unwrap().as_bool().unwrap(),
            humidity: measurement
                .get("humidity")
                .unwrap()
                .as_n()
                .unwrap()
                .to_string(),
            media_month: *measurement.get("media_month").unwrap().as_bool().unwrap(),
            soil_humidity: measurement
                .get("soil_humidity")
                .unwrap()
                .as_n()
                .unwrap()
                .to_string(),
            temperature: measurement
                .get("temperature")
                .unwrap()
                .as_n()
                .unwrap()
                .to_string(),
        })
    }
    list_of_measurements
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    let user_id = get_user_id(&event);

    let measuration = hashmap_to_lists(get_list(&client, &user_id).await.items().unwrap().to_vec());

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(serde_json::to_string(&measuration).unwrap().into())  // Serialize response to string
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
