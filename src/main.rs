use std::collections::HashMap;

use crate::parser::parse;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use chrono::Utc;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
pub mod parser;

struct AppData {
    cookie: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestQuery {
    date: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://www.easistent.com/nadomescanja/30a1b45414856e5598f2d137a5965d5a4ad36826")
        .send()
        .await?;

    let cookie = response
        .headers()
        .get("set-cookie")
        .ok_or("Cookie not present")?
        .to_str()
        .unwrap_or_default()
        .to_string();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppData {
                cookie: cookie.clone(),
            }))
            .service(substitutions)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}

#[get("/")]
async fn substitutions(
    _: HttpRequest,
    data: web::Data<AppData>,
    query: web::Query<RequestQuery>,
) -> Result<HttpResponse> {
    let client = reqwest::Client::new();

    let date_now = Utc::now().to_string();

    let date = match &query.date {
        Some(d) => d,
        None => date_now.split(" ").collect::<Vec<&str>>()[0],
    };

    let mut split = date.split("-").collect::<Vec<&str>>();
    split.reverse();

    let mut params = HashMap::new();
    params.insert("datum", split.join("."));
    params.insert("id_sola", "182".to_string());

    match client
        .post("https://www.easistent.com/organizacija/ajax_spremeni_seznam_nadomescanj")
        .header("cookie", &data.cookie)
        .form(&params)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()),
    };

    let response = match client
        .get("https://www.easistent.com/nadomescanja/30a1b45414856e5598f2d137a5965d5a4ad36826")
        .header("cookie", &data.cookie)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()),
    };

    let html = match response.text().await {
        Ok(r) => r,
        Err(_) => return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()),
    };

    if let Ok(sub) = parse(html) {
        return Ok(HttpResponse::build(StatusCode::OK).json(sub));
    }
    return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish());
}
