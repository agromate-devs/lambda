use super::models::PostRequest;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use aws_sdk_iotdataplane as iotdataplane;
use aws_sdk_iotdataplane::primitives::Blob;
use helper::get_user_id;

const TABLE_NAME: &str = "plants"; // DynamoDB table name
const SNS_ARN: &str = "arn:aws:sns:eu-central-1:284535252702:app/GCM/sensor_notification";
const SNS_ARN_TABLE: &str = "notification_devices"; // Collection of all notification registred devices

async fn filter_uid(client: Client, uid: &str) -> Result<aws_sdk_dynamodb::operation::query::QueryOutput, aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::query::QueryError>>
{
    client
        .query()
        .table_name(TABLE_NAME)
        .key_condition_expression("user_id = :id")
        .expression_attribute_values(":id", AttributeValue::S(uid.to_string()))
        .send()
        .await
}

async fn filter_uid_and_sid(client: Client, uid: &str, sensor_id: &str) -> Result<aws_sdk_dynamodb::operation::query::QueryOutput, aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::query::QueryError>>
{
    client
    .query()
    .table_name(TABLE_NAME)
    .key_condition_expression("user_id = :id and sensor_id = :sid")
    .expression_attribute_values(":id", AttributeValue::S(uid.to_string()))
    .expression_attribute_values(":sid", AttributeValue::S(sensor_id.to_string()))
    .send()
    .await
}

pub async fn get_plant(client: Client, uid: &str, sensor_id: &str) -> Result<String, String> {
    let results = if sensor_id != "NULL" {filter_uid_and_sid(client, uid, sensor_id).await} else {filter_uid(client, uid).await};

    match results {
        Ok(results) => {
            let plants: Vec<PostRequest> =
                results.items().unwrap().iter().map(|v| v.into()).collect();
            let plants_str: String = serde_json::to_string(&plants).unwrap();
            Ok(plants_str)
        }
        Err(err) => 
        {
            println!("{:?}", err.raw_response());            
            Err("Something went wrong while finding plant in DB".to_string())
        }
    }
}

async fn add_device_to_notification(device_token: String, uid: String, sns_client: aws_sdk_sns::Client, dynamodb_client: Client) {
    let result = sns_client.create_platform_endpoint()
        .platform_application_arn(SNS_ARN)
        .token(device_token)
        .send()
        .await.expect("Error adding device to SNS");

        dynamodb_client.put_item()
            .table_name(SNS_ARN_TABLE)
            .item("user_id", AttributeValue::S(uid))
            .item("arn", AttributeValue::S(result.endpoint_arn.unwrap()))
            .send().await.expect("Error adding device notification to DynamoDB");
}

pub async fn add_plant(dynamodb_client: Client, iot_data_client: aws_sdk_iotdataplane::Client, sns_client: aws_sdk_sns::Client, request: PostRequest) -> Result<String, String> {
    let iot_request = request.clone();
    let response = dynamodb_client // Save all details of plant in dynamoDB so our ESP8266 can use it
        .put_item()
        .table_name(TABLE_NAME)
        .item("user_id", AttributeValue::S(request.clone().user_id))
        .item("plant_name", AttributeValue::S(request.plant_name))
        .item("sensor_id", AttributeValue::S(request.sensor_id.clone()))
        // Start Temperature
        .item(
            "default_temperature",
            AttributeValue::N(request.default_temperature.to_string()),
        )
        .item(
            "temperature_limit",
            AttributeValue::N(request.temperature_limit.to_string()),
        )
        .item(
            "notify_wrong_temperaure",
            AttributeValue::Bool(request.notify_wrong_temperature),
        )
        // End Temperature
        // Start Humidity
        .item(
            "default_humidity",
            AttributeValue::N(request.default_humidity.to_string()),
        )
        .item(
            "humidity_limit",
            AttributeValue::N(request.humidity_limit.to_string()),
        )
        .item(
            "notify_wrong_humidity",
            AttributeValue::Bool(request.notify_wrong_humidity),
        )
        // End humidity
        // Start Precipitation
        .item(
            "default_precipitation",
            AttributeValue::N(request.default_precipitation.to_string()),
        )
        .item(
            "precipitation_limit",
            AttributeValue::N(request.precipitation_limit.to_string()),
        )
        .item(
            "notify_wrong_soil_humidity",
            AttributeValue::Bool(request.notify_wrong_soil_humidity),
        )
        // End precipitation
        // Start color
        .item(
            "default_light_color",
            AttributeValue::S(request.default_light_color),
        )
        .item(
            "light_time",
            AttributeValue::N(request.light_time.to_string()),
        )
        .item(
            "light_intensiy",
            AttributeValue::N(request.light_intensity.to_string()),
        )
        // End color
        .send()
        .await;
    
    iot_data_client.publish().topic(format!("sensor/plants/{}", request.sensor_id)).payload(Blob::new(serde_json::to_string(&iot_request).unwrap())).send().await.expect("Error in MQTT publish"); 

    if request.notify_wrong_humidity || request.notify_wrong_temperature || request.notify_wrong_soil_humidity {
        add_device_to_notification(request.device_token, request.user_id, sns_client, dynamodb_client).await;
    }
    
    match response {
        Ok(_) => Ok("Plant added correctly".to_string()),
        Err(_) => Err("Error during adding plant".to_string()),
    }
}
