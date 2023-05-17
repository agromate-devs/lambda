use aws_config::load_from_env;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::{iter::Iterator, collections::HashMap};
use tokio_stream::StreamExt;
use chrono::Utc;

#[derive(Deserialize)]
struct Request {
    // command: String,
}

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

async fn get_all_items(client: &Client, table_name: &str) -> Result<Vec<HashMap<String, AttributeValue>>, Error> {
    let items: Result<Vec<_>, _> = client
    .scan()
    .table_name(table_name)
    .into_paginator()
    .items()
    .send()
    .collect()
    .await;
    match items {
        Ok(items_parsed) => Ok(items_parsed),
        Err(err) => Err(err.into()),
    }
}

async fn delete_item(client: &Client, table_name: &str, value: String) -> bool{
    let response = client
    .delete_item()
    .table_name(table_name)
    .key(
        "timestamp",
        AttributeValue::N(
            value
        ),
    )
    .send()
    .await;
    match response {
        Ok(_response) => true,
        Err(err) => {
            println!("{:?}", err);
            false
        }
    }
}
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let shared_config = load_from_env().await;
    let client = Client::new(&shared_config);

    let req = client.list_tables();
    let tables = req.send().await.unwrap();
    for table in tables.table_names().unwrap() {
        let items = get_all_items(&client, table).await.unwrap();
        let items_filtered: Vec<_> = items
            .clone()
            .into_iter()
            .filter(|x| x["hour"] == AttributeValue::Bool(true))
            .collect();
        if items_filtered.len() == 24 {
            let mut temp_media: f32 = 0.0;
            let mut hum_media: f32 = 0.0;
            for item in items_filtered {
                temp_media += item["temperature"].as_n().unwrap().parse::<f32>().unwrap();
                hum_media += item["humidity"].as_n().unwrap().parse::<f32>().unwrap();
                delete_item(&client, table, item["timestamp"].as_n().unwrap().to_string()).await;
            }
            client.put_item()
            .table_name(table)
            .item("media_temp", AttributeValue::N(temp_media.to_string()))
            .item("media_hum", AttributeValue::N(hum_media.to_string()))
            .item("hour", AttributeValue::Bool(false))
            .item("media_month", AttributeValue::Bool(true))
            .item("media_year", AttributeValue::Bool(false))
            .item("timestamp", AttributeValue::N(Utc::now().timestamp().to_string())).send().await.unwrap();
        }
        let items_media_filtered: Vec<_> = items.clone()
        .into_iter()
        .filter(|x| x["media_month"] == AttributeValue::Bool(true)).collect();
        if items_media_filtered.len() == 11 {   // one month is missing since we create now
            let mut media_temp_month = 0.0;
            let mut media_hum_month = 0.0;
            for media_month in items_media_filtered {
                media_temp_month += media_month["media_temp"].as_n().unwrap().parse::<f32>().unwrap();
                media_hum_month += media_month["media_hum"].as_n().unwrap().parse::<f32>().unwrap();
            }
            client.put_item()
            .table_name(table)
            .item("media_temp", AttributeValue::N(media_temp_month.to_string()))
            .item("media_hum", AttributeValue::N(media_hum_month.to_string()))
            .item("hour", AttributeValue::Bool(false))
            .item("media_year", AttributeValue::Bool(true))
            .item("media_month", AttributeValue::Bool(false))
            .item("timestamp", AttributeValue::N(Utc::now().timestamp().to_string())).send().await.unwrap();
        }
    }

    // Prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("Command."),
    };

    // Return `Response` (it will be serialized to JSON automatically by the runtime)
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