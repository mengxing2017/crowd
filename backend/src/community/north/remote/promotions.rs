use crate::{
    Application,
    community::domain::Failure,
    http::{request_id, response, token},
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};

pub(super) async fn resolve(State(app): State<Application>, Path(code): Path<String>) -> Response {
    let r = request_id();
    response(app.community.resolve(&code).await, &r)
}

pub(super) async fn qr(
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
