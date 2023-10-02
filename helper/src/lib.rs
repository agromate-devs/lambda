use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation, TokenData};
use lambda_http::Request;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct UserJWT {    // Minimal JWT structure with only the data that we need
    user_id: String
}

pub fn get_user_id(request: &Request) -> String {
    let jwt = request.headers().get("authorization").unwrap().to_str().expect("Parse JWT failed");    // Parse authorization header
    let key = DecodingKey::from_secret(&[]);    // Create an empty secret since we don't need to validate JWT
    let mut validation = Validation::new(Algorithm::HS256);
    validation.insecure_disable_signature_validation(); // Validation is made by API Gateway already
    if cfg!(debug_assertions) { // In the test environment, we use an expired JWT for obvious reasons
        validation.validate_exp = false;
    }
    let data: TokenData<UserJWT> = decode(jwt, &key, &validation).expect("Decode JWT failed"); // Decode JWT and get only user_id
    data.claims.user_id
}
