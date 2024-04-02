use crate::common::error::AppError;
use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, MatchedPath, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    RequestPartsExt,
};
use serde::Serialize;
use serde_json::{json, Value};

// #[derive(FromRequest)]
// #[from_request(via(axum::Json), rejection(AppError))]
// pub struct AppJson<T>(pub T);

// impl<T> IntoResponse for AppJson<T>
// where
//     axum::Json<T>: IntoResponse,
// {
//     fn into_response(self) -> axum::response::Response {
//         axum::Json(self.0).into_response()
//     }
// }
// #[derive(FromRequest)]
// #[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for AppJson<T>
where
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<Value>);

    async fn from_request(
        req: Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();

        // We can use other extractors to provide better rejection messages.
        // For example, here we are using `axum::extract::MatchedPath` to
        // provide a better error message.
        //
        // Have to run that first since `Json` extraction consumes the request.
        let path = parts
            .extract::<MatchedPath>()
            .await
            .map(|path| path.as_str().to_owned())
            .ok();

        let req = Request::from_parts(parts, body);

        match axum::Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            // convert the error from `axum::Json` into whatever we want
            Err(rejection) => {
                let payload = json!({
                    "message": rejection.body_text(),
                    "origin": "custom_extractor",
                    "path": path,
                });

                Err((rejection.status(), axum::Json(payload)))
            }
        }
    }
}

impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

#[derive(Serialize)]
#[serde(untagged)] // 使用未标记的联合，以便在序列化时根据实际类型选择适当的结构体
pub enum RespResult<T> {
    Success(Success<T>),
    Error(Error),
}

#[derive(Serialize)]
pub struct Success<T> {
    pub code: u64,
    pub message: String,
    pub data: T,
}

#[derive(Serialize)]
pub struct Error {
    pub code: u64,
    pub message: String,
}

// 构建成功响应
pub fn build_success_response<T>(data: T) -> RespResult<T> {
    RespResult::Success(Success {
        code: 200,
        message: "Operation successful".to_string(),
        data,
    })
}

// 构建错误响应
pub fn build_error_response(message: String) -> RespResult<()> {
    RespResult::Error(Error { code: 9999, message: message })
}
