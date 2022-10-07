use crate::parser::parse;
use actix_web::{get, App, HttpResponse, HttpServer, Result};
use reqwest::StatusCode;
pub mod parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    HttpServer::new(|| App::new().service(substitutions))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}

#[get("/")]
async fn substitutions() -> Result<HttpResponse> {
    let client = reqwest::Client::new();

    let response = match client
        .get("https://www.easistent.com/nadomescanja/30a1b45414856e5598f2d137a5965d5a4ad36826")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()),
    };

    let html = match response.text().await {
        Ok(r) => r,
        Err(e) => return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()),
    };

    if let Ok(sub) = parse(html) {
        return Ok(HttpResponse::build(StatusCode::OK).json(sub));
    }
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish());
}
