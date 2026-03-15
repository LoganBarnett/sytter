use crate::state::{State, SytterVariable};
use actix_web::{
  http::header,
  web::{self, Data},
  App, HttpRequest, HttpResponse, HttpServer,
};
use futures::TryFutureExt;
use serde::Serialize;
use tracing::*;

use crate::error::AppError;

fn data_to_text(data: &Vec<SytterVariable>) -> String {
  data
    .into_iter()
    .map(|v| format!("{}={}", v.key, v.value))
    .collect::<Vec<String>>()
    .join("\n")
}

pub async fn index(req: HttpRequest) -> Result<HttpResponse, AppError> {
  let data = State::get_variables();
  Ok(
    HttpResponse::Ok().body(
      match req
        .headers()
        .get(header::ACCEPT)
        .map(|h| {
          h.to_str()
            // Is there a better way to do this?
            .unwrap_or("invalid-header-value".into())
        })
        .unwrap_or("application/text")
      {
        "application/text" => data_to_text(&data),
        "application/json" => serdeconv::to_json_string(&data)
          .map_err(AppError::HttpJsonSerializeError)?
          .into(),
        _ => data_to_text(&data),
      },
    ),
  )
}

pub async fn upsert(
  payload: web::Json<SytterVariable>,
) -> Result<HttpResponse, AppError> {
  State::set_variable(payload.to_owned());
  Ok(HttpResponse::Ok().finish())
}

/// Health check response structure following standard REST API conventions.
/// Returns HTTP 200 with JSON body containing status and version.
#[derive(Serialize)]
struct HealthResponse {
  status: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  version: Option<String>,
}

/// Health check endpoint handler.
///
/// Provides a standard HTTP health check endpoint following common conventions:
/// - `/health` - Common REST API convention
/// - `/healthz` - Kubernetes/Google convention (z suffix avoids collisions)
///
/// Returns HTTP 200 OK with JSON body:
/// ```json
/// {
///   "status": "ok",
///   "version": "0.1.0"
/// }
/// ```
///
/// This is a basic liveness check that indicates the HTTP server is responding.
/// Future enhancements could include:
/// - Database connectivity checks.
/// - Trigger health status.
/// - System resource checks.
pub async fn health() -> HttpResponse {
  let response = HealthResponse {
    status: "ok".to_string(),
    version: Some(env!("CARGO_PKG_VERSION").to_string()),
  };
  HttpResponse::Ok().json(response)
}

/// Start the HTTP server with the configured port.
///
/// Provides the following endpoints:
/// - `GET /health` - Health check endpoint (standard REST convention).
/// - `GET /healthz` - Health check endpoint (Kubernetes convention).
/// - `GET /state` - Get all state variables.
/// - `POST /state` - Set/update a state variable.
pub async fn http_server(port: usize) -> Result<(), AppError> {
  info!("HTTP server starting on port {}...", port);
  HttpServer::new(move || {
    App::new()
      // Health check endpoints - support both common conventions.
      .service(web::resource("/health").get(health))
      .service(web::resource("/healthz").get(health))
      // State management endpoints.
      .service(web::resource("/state").get(index).post(upsert))
  })
  .bind(("0.0.0.0", port as u16))
  .map_err(AppError::HttpBindError)?
  .run()
  .map_err(AppError::HttpStartError)
  .await
}
