use lambda_http::{ Response, Body, Error };

pub fn success_response() -> Result<Response<Body>, Error>{
    let resp = Response::builder()
    .status(200)
    .header("content-type", "text/html")
    .body("OK".into())
    .map_err(Box::new)
    .unwrap();
    Ok(resp) 
}

pub fn internal_server_error(endpoint: &str) -> Result<Response<Body>, Error>{
    let resp = Response::builder()
    .status(500)
    .header("content-type", "text/html")
    .body(format!("Internal Server Error in {}", endpoint).into())
    .map_err(Box::new)
    .unwrap();
    Ok(resp) 
}