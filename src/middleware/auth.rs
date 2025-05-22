use axum::{
    body::Body, http::{header, Request, StatusCode}, middleware::Next, response::Response
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user id
    pub exp: usize,   // expiration time
}

pub async fn auth(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for registration and login
    let path = req.uri().path();
    if path == "/api/v1/user/register" || path == "/api/v1/user/login" || path == "/" {
        return Ok(next.run(req).await);
    }

    // Print all headers for debugging
    println!("Received headers:");
    for (name, value) in req.headers() {
        println!("  {}: {}", name, value.to_str().unwrap_or("<invalid>"));
    }
    
    // Get the authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or_else(|| {
            println!("headers: {:?}", req.headers().keys());
            let has = req.headers().contains_key(header::AUTHORIZATION);
            println!("has: {:?}", has);
            println!("No authorization header found");
            StatusCode::UNAUTHORIZED
        })?
        .to_str()
        .map_err(|_| {
            println!("Invalid authorization header value");
            StatusCode::UNAUTHORIZED
        })?;

    // Extract the token from the Bearer header
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            println!("No Bearer token found in authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    // Decode and validate the token
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY is not defined").as_bytes()),
        &Validation::default(),
    )
    // .unwrap()
    .map_err(|_| {
        println!("Invalid token");
        StatusCode::UNAUTHORIZED
    })?.claims;

    // Add the user ID to the request extensions
    let mut req = req;
    req.extensions_mut().insert(claims.sub);

    Ok(next.run(req).await)
}