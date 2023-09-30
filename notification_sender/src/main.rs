use aws_config::load_from_env;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Serialize, Deserialize};
use tracing_subscriber::fmt::format;
use std::collections::HashMap;

const SNS_DEVICES_TABLE: &str = "notification_devices";

/*
    {
        "notification": {
            "title":"This is the needed title for system display",
            "body":"This is the needed body for system display"
        },
        "data" : {
            "custom_app_field1" : "Whatever",
            "custom_app_field2" : "Whenever"
        }
    }
*/

#[derive(Debug, Serialize)]
struct Notification {
    title: String,
    body: String,
}

#[derive(Debug, Serialize)]
struct Data {
    custom_app_field1: String,
    custom_app_field2: String,
}

#[derive(Debug, Serialize)]
struct SNSNotification {
    notification: Notification,
    data: Data,
}

#[derive(Debug, Serialize)]
struct SNSProtocolMessage {
    GCM: String
}

/*

    Example MQTT message:

    {
        "user_id": "WLk7Giku6TYBMI22wfmTSJbWOVA2",
        "is_temperature_notification": true,
        "is_humidity_notification": true,
        "temperature": 24,
        "humidity": 63,
        "soil_humidity": 0,
        "hour": true,
        "media_month": false,
        "uuid": "297a0620-3b4d-40ed-b407-2216eb0d"
    }

*/

#[derive(Deserialize, Clone)]
struct Request {
    user_id: String,
    is_temperature_notification: bool,
    is_humidity_notification: bool,
    temperature: i8,
    humidity: i8,
    soil_humidity: i8,
    hour: bool,
    media_month: bool,
    uuid: String,
}

fn create_notification_temperature(mqtt_message: Request) -> SNSProtocolMessage {
    let notification_title = if mqtt_message.is_temperature_notification && mqtt_message.is_humidity_notification {
        "Agromate avvisi sensori"
    } else if mqtt_message.is_temperature_notification {
        "Agromate avviso temperatura"
    } else if mqtt_message.is_humidity_notification {
        "Agromate avviso umidità"
    } else {""}.to_string();

    let notification_body = if mqtt_message.is_temperature_notification && mqtt_message.is_humidity_notification {
        "I sensori di umidità e temperatura indicano valore inadeguati per la tua pianta! Controlla la serra e mettila in un luogo più fresco".to_string()
    } else if mqtt_message.is_temperature_notification {
        format!("La temperature è fuori dal range, temperatura attuale: {}", mqtt_message.temperature)
    } else if mqtt_message.is_humidity_notification {
        format!("L'umidità è fuori dal range, umidità attuale: {}", mqtt_message.humidity)
    } else {"".to_string()};

    let notification = SNSNotification {
        notification: Notification {
            title: notification_title,
            body: notification_body,
        },
        data: Data {
            custom_app_field1: "not_used".to_string(),
            custom_app_field2: "not_used".to_string(),
        },
    };
    SNSProtocolMessage {
        GCM: serde_json::to_string(&notification).unwrap()
    }
}


#[derive(Debug, Clone)]
struct NotificationDevice {
    arn: String,
}

impl Into<NotificationDevice> for &HashMap<String, AttributeValue> {
    fn into(self) -> NotificationDevice {
        NotificationDevice {
            arn: self.get("arn").unwrap().as_s().unwrap().to_string(),
        }
    }
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<(), Error> {
    println!("OK");
    let shared_config = load_from_env().await;
    let client = aws_sdk_sns::Client::new(&shared_config);
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);

    let results = dynamodb_client
        .query()
        .table_name(SNS_DEVICES_TABLE)
        .key_condition_expression("#user_id = :uid")
        .expression_attribute_names("#user_id", "user_id")
        .expression_attribute_values(":uid", AttributeValue::S(event.clone().payload.user_id))
        .send()
        .await?;

    if let Some(items) = results.items {
        let notification_devices: Vec<NotificationDevice> = items
            .iter()
            .map(|v: &HashMap<String, AttributeValue>| v.into())
            .collect();
        if notification_devices.len() == 0 {
            panic!("No devices were found");
        }
        let payload = create_notification_temperature(event.payload);

        let _ = client
            .publish()
            .target_arn(notification_devices[0].arn.clone())
            .message_structure("json")
            .message(serde_json::to_string(&payload).expect("Serialization failed"))
            .send()
            .await
            .unwrap();
    } else {
        panic!("No devices were found");
    }

    Ok(())
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
