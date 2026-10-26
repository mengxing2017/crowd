use crate::{
    Application,
    community::{domain::Failure, south::adapter::wechat::WechatIdentity},
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{
        DefaultBodyLimit, Path, Query, State,
        rejection::{BytesRejection, JsonRejection},
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};
static REQUEST: AtomicU64 = AtomicU64::new(1);
fn request_id() -> String {
    format!(
        "{:x}-{:x}-{:x}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        REQUEST.fetch_add(1, Ordering::Relaxed)
    )
}
fn token(headers: &HeaderMap) -> &str {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("")
}
fn response(result: Result<Value, Failure>, request: &str) -> Response {
    let (status, value) = match result {
        Ok(v) => (StatusCode::OK, json!({"data":v,"request_id":request})),
        Err(Failure(code, message)) => {
            let status = match code {
                "SESSION_EXPIRED" | "AUTH_INVALID" => StatusCode::UNAUTHORIZED,
                "FORBIDDEN" => StatusCode::FORBIDDEN,
                "NOT_FOUND" => StatusCode::NOT_FOUND,
                "VERSION_CONFLICT" | "IDEMPOTENCY_CONFLICT" => StatusCode::CONFLICT,
                "SERVICE_UNAVAILABLE"
                | "IDENTITY_UNAVAILABLE"
                | "DEPENDENCY_UNAVAILABLE"
                | "AGREEMENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::UNPROCESSABLE_ENTITY,
            };
            (
                status,
                json!({"error":{"code":code,"message":message,"request_id":request}}),
            )
        }
    };
    (
        status,
        [
            ("cache-control", "no-store"),
            ("x-content-type-options", "nosniff"),
        ],
        Json(value),
    )
        .into_response()
}
fn parse(body: Result<Json<Value>, JsonRejection>) -> Result<Value, Failure> {
    body.map(|v| v.0)
        .map_err(|_| Failure("INVALID_INPUT", "请求必须是合法 JSON 且未超过处理上限"))
}
pub fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/community/agreements", get(agreements))
        .route("/api/v1/community/login", post(login))
        .route("/api/v1/community/admin/login", post(staff_login))
        .route("/api/v1/community/session", post(redeem))
        .route("/api/v1/community/resolve/{code}", get(resolve))
        .route("/api/v1/community/read/{section}", get(read))
        .route("/api/v1/community/commands/{action}", post(command))
        .route("/api/v1/community/files/{id}", get(file).post(upload))
        .route("/api/v1/community/qr/{code}", get(qr))
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024))
}
async fn agreements(State(app): State<Application>) -> Response {
    let r = request_id();
    response(app.community.agreements().await, &r)
}
async fn login(
    State(app): State<Application>,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    let r = request_id();
    let result = match parse(body) {
        Ok(input) => app.community.login(&WechatIdentity, &input, &r).await,
        Err(e) => Err(e),
    };
    response(result, &r)
}
async fn resolve(State(app): State<Application>, Path(code): Path<String>) -> Response {
    let r = request_id();
    response(app.community.resolve(&code).await, &r)
}
#[derive(Deserialize)]
struct Scope {
    target: Option<String>,
}
async fn read(
    State(app): State<Application>,
    Path(section): Path<String>,
    Query(scope): Query<Scope>,
    headers: HeaderMap,
) -> Response {
    let r = request_id();
    response(
        app.community
            .read(token(&headers), &section, scope.target.as_deref(), &r)
            .await,
        &r,
    )
}
async fn command(
    State(app): State<Application>,
    Path(action): Path<String>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    let r = request_id();
    let key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty() && s.len() <= 128);
    let result = match (key, parse(body)) {
        (Some(key), Ok(input)) => {
            app.community
                .command(token(&headers), &action, &input, key, &r)
                .await
        }
        (None, _) => Err(Failure("INVALID_INPUT", "缺少有效 Idempotency-Key")),
        (_, Err(e)) => Err(e),
    };
    response(result, &r)
}
async fn upload(
    State(app): State<Application>,
    Path(purpose): Path<String>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    let r = request_id();
    let result = match body {
        Ok(bytes) => {
            app.community
                .upload(token(&headers), &purpose, bytes.to_vec(), &r)
                .await
        }
        Err(_) => Err(Failure(
            "INVALID_INPUT",
            "图片请求超过服务处理上限或读取失败",
        )),
    };
    response(result, &r)
}
async fn file(
    State(app): State<Application>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let r = request_id();
    match app.community.file(token(&headers), &id, &r).await {
        Ok((mime, bytes)) => (
            [
                ("content-type", mime),
                ("cache-control", "no-store".into()),
                ("x-content-type-options", "nosniff".into()),
            ],
            bytes,
        )
            .into_response(),
        Err(e) => response(Err(e), &r),
    }
}
async fn qr(
    State(app): State<Application>,
    Path(code): Path<String>,
    headers: HeaderMap,
) -> Response {
    let r = request_id();
    let result = app.community.read(token(&headers), "codes", None, &r).await;
    match result {
        Ok(v)
            if v.as_array().is_some_and(|v| {
                v.iter()
                    .any(|c| c["id"] == code && c["code"]["active"] == true)
            }) =>
        {
            let origin = match std::env::var("CROWD_PROMOTION_ORIGIN") {
                Ok(v) if v.starts_with("https://") && !v.contains(['?', '#']) => {
                    v.trim_end_matches('/').to_owned()
                }
                _ => {
                    return response(
                        Err(Failure("DEPENDENCY_UNAVAILABLE", "未配置 HTTPS 推广入口")),
                        &r,
                    );
                }
            };
            let payload = format!("{origin}/?source={code}");
            let qr = match qrcode::QrCode::new(payload.as_bytes()) {
                Ok(v) => v,
                Err(_) => return response(Err(Failure("INVALID_INPUT", "推广入口过长")), &r),
            };
            let width = qr.width();
            let scale = 8;
            let border = 4;
            let size = (width + border * 2) * scale;
            let mut image =
                image::GrayImage::from_pixel(size as u32, size as u32, image::Luma([255]));
            for y in 0..width {
                for x in 0..width {
                    if qr[(x, y)] == qrcode::Color::Dark {
                        for dy in 0..scale {
                            for dx in 0..scale {
                                image.put_pixel(
                                    ((x + border) * scale + dx) as u32,
                                    ((y + border) * scale + dy) as u32,
                                    image::Luma([0]),
                                );
                            }
                        }
                    }
                }
            }
            let mut output = std::io::Cursor::new(Vec::new());
            if image
                .write_to(&mut output, image::ImageFormat::Png)
                .is_err()
            {
                return response(Err(Failure("SERVICE_UNAVAILABLE", "二维码暂不可生成")), &r);
            }
            (
                [("content-type", "image/png"), ("cache-control", "no-store")],
                output.into_inner(),
            )
                .into_response()
        }
        Ok(_) => response(Err(Failure("FORBIDDEN", "无权获取此推广码")), &r),
        Err(e) => response(Err(e), &r),
    }
}

async fn redeem(
    State(app): State<Application>,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    let r = request_id();
    let result = match parse(body) {
        Ok(v) => match crate::community::domain::text(&v, "ticket") {
            Ok(ticket) => app.community.repo.redeem(ticket).await,
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    };
    response(result, &r)
}

async fn staff_login(
    State(app): State<Application>,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    let r = request_id();
    let result = match parse(body) {
        Ok(input) => app.community.staff_login(&input, &r).await,
        Err(e) => Err(e),
    };
    response(result, &r)
}
