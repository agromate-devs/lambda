use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::{ AttributeDefinition, ScalarAttributeType, KeySchemaElement, KeyType, BillingMode, AttributeValue};
use serde::{Deserialize, Serialize};
use chrono::Utc;

// const TRACKER: &str = "TRACKER";

#[derive(Deserialize)]
struct Request {
    temperature: i8,
    humidity: i8,
    uuid: String,
    hour: bool,
    media_month: bool
}

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

async fn table_exists(client: &Client, table_name: String) -> bool {
    let req = client.list_tables().limit(10);
    let resp = req.send().await.unwrap();
    resp.table_names().unwrap().contains(&table_name)
}

async fn create_table(client: &Client, table_name: &str, key: &str) -> bool {
    let key_name: String = key.into();

    let ad = AttributeDefinition::builder()
    .attribute_name(key_name.clone())
    .attribute_type(ScalarAttributeType::N)
    .build();

    let ks = KeySchemaElement::builder()
        .attribute_name(key_name)
        .key_type(KeyType::Hash)
        .build();

    
    let req = client.create_table()
    .billing_mode(BillingMode::PayPerRequest)
    .table_name(table_name)
    .key_schema(ks)
    .attribute_definitions(ad)
    .send()
    .await;

    match req {
        Ok(_out) => {
            // println!("Created table: {:?}", out);
            return true;
        }
        Err(err) => {
            println!("Error creating table: {:?}", err);
            return false;
        }
    }

}

async fn insert_into(client: Client, 
                table_name: String, 
                key: &str, 
                value: i8,
                key2: &str,
                value2: i8, 
                key3: &str, 
                value3: bool,
                key4: &str,
                value4: bool
    ) -> bool {
    let temp = AttributeValue::N(value.to_string());
    let hum = AttributeValue::N(value2.to_string());
    let dt = Utc::now();
    let timestamp: i64 = dt.timestamp();
    let timestamp = AttributeValue::N(timestamp.to_string());
    let request = client.put_item()
    .table_name(table_name)
    .item(key, temp)
    .item(key2, hum)
    .item("timestamp", timestamp)
    .item(key3, AttributeValue::S(value3.to_string()))
    .item(key4, AttributeValue::S(value4.to_string()));

    let response = request.send().await;
    match response {
        Ok(_ok) => true,
        Err(err) => {
            println!("{:?}", err);
            false
        }
    }
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);
    let uuid: String = event.payload.uuid;

    if !table_exists(&client, uuid.clone()).await{
        create_table(&client, &uuid, "timestamp").await;
    }
    
    let temp = event.payload.temperature;
    let humidity = event.payload.humidity;
    let hour = event.payload.hour;
    let media_month = event.payload.media_month;

    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("{}", insert_into(client,
            uuid, 
            "temperature", temp, 
            "humidity", humidity,
            "hour", hour,
            "media_month", media_month).await),
    };

    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
