use crate::{
    Application,
    http::{request_id, response},
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};

pub(super) async fn product_image(
    State(app): State<Application>,
    Path(id): Path<String>,
) -> Response {
    let r = request_id();
    match app.community.product_image(&id).await {
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
