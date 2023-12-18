use std::sync::{Arc, Mutex};

use log::*;
use crate::state::{State, SytterVariable};
use actix_web::{
  http::header, web::{self, Data}, App, HttpRequest, HttpResponse, HttpServer
};
use futures::TryFutureExt;

use crate::error::AppError;

fn data_to_text(data: &Vec<SytterVariable>) -> String {
  data
    .into_iter()
    .map(|v| format!("{}={}", v.key, v.value) )
    .collect::<Vec<String>>()
    .join("\n")
}

pub async fn index(req: HttpRequest) -> Result<HttpResponse, AppError> {
  let data = State::get_variables();
  Ok(
    HttpResponse::Ok()
      .body(
        match req
          .headers()
          .get(header::ACCEPT)
          .map(|h| {
            h
              .to_str()
              // Is there a better way to do this?
              .unwrap_or("invalid-header-value".into())
          })
          .unwrap_or("application/text") {
            "application/text" => data_to_text(&data),
            "application/json" => serdeconv::to_json_string(&data)
              .map_err(AppError::HttpJsonSerializeError)
              ?
              .into(),
            _ => data_to_text(&data),
          }
      )
  )
}

pub async fn upsert(
  payload: web::Json<SytterVariable>,
) -> Result<HttpResponse, AppError> {
  State::set_variable(payload.to_owned());
  Ok(HttpResponse::Ok().finish())
}

pub async fn http_server() -> Result<(), AppError> {
  info!("HTTP server starting...");
  HttpServer::new(move || {
    App::new()
      .service(
        web::resource("/state")
          .get(index)
          .post(upsert)
      )

  })
    .bind(("0.0.0.0", 8080))
    .map_err(AppError::HttpBindError)
    ?
    .run()
    .map_err(AppError::HttpStartError)
    .await
}
