use std::collections::HashMap;
use aws_sdk_dynamodb::types::AttributeValue;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PostRequest {
    #[serde(skip)]  // We don't get this from request body but from JWT
    pub user_id: String,    // Firebase user ID
    pub plant_name: String,
    pub sensor_id: String,  // UUID of ESP8266
    pub device_token: String,   // FCM device token
    pub default_temperature: f32,
    pub temperature_limit: f32,
    pub notify_wrong_temperature: bool,
    pub default_humidity: f32,
    pub humidity_limit: f32,
    pub notify_wrong_humidity: bool,
    pub default_precipitation: f32,
    pub precipitation_limit: f32,
    pub notify_wrong_soil_humidity: bool,
    pub default_light_color: String,
    pub light_time: f32, // In hour, minute
    pub light_intensity: i8
}

fn AttributeValueToString(value: AttributeValue) -> String {
    value.as_s().unwrap().to_owned()
}

fn AttributeValueToFloat(value: AttributeValue) -> f32 {
    value.as_n().unwrap().to_owned().parse::<f32>().unwrap()
}

fn AttributeValueToInt(value: AttributeValue) -> i8 {
    value.as_n().unwrap().to_owned().parse::<i8>().unwrap()
}

fn AttributeValueToBoolean(value: AttributeValue) -> bool { 
    value.as_bool().unwrap().to_owned()
}

impl From<&HashMap<String, AttributeValue>> for PostRequest {
    fn from(value: &HashMap<String, AttributeValue>) -> Self {
        PostRequest {
            user_id: AttributeValueToString(value.get("user_id").unwrap().clone()) ,    // Firebase user ID
            plant_name: AttributeValueToString(value.get("plant_name").unwrap().clone()),
            sensor_id: AttributeValueToString(value.get("sensor_id").unwrap().clone()),  // UUID of ESP8266
            device_token: "".to_string(),   // FCM device token, we don't use it in this DynamoDB table
            // device_token: AttributeValueToString(value.get("device_token").unwrap().clone()),   // FCM device token
            default_temperature: AttributeValueToFloat(value.get("default_temperature").unwrap().clone()),
            temperature_limit: AttributeValueToFloat(value.get("temperature_limit").unwrap().clone()),
            notify_wrong_temperature: AttributeValueToBoolean(value.get("notify_wrong_temperature").unwrap().clone()),
            default_humidity: AttributeValueToFloat(value.get("default_humidity").unwrap().clone()),
            humidity_limit: AttributeValueToFloat(value.get("humidity_limit").unwrap().clone()),
            notify_wrong_humidity: AttributeValueToBoolean(value.get("notify_wrong_humidity").unwrap().clone()),
            default_precipitation:AttributeValueToFloat(value.get("default_precipitation").unwrap().clone()) ,
            precipitation_limit: AttributeValueToFloat(value.get("precipitation_limit").unwrap().clone()),
            notify_wrong_soil_humidity: AttributeValueToBoolean(value.get("notify_wrong_soil_humidity").unwrap().clone()),
            default_light_color: AttributeValueToString(value.get("default_light_color").unwrap().clone()),
            light_time: AttributeValueToFloat(value.get("light_time").unwrap().clone()) , // In hour, minute
            light_intensity: AttributeValueToInt(value.get("default_temperature").unwrap().clone())
        }
    }
}