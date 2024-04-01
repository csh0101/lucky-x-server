// use axum::Json;
// pub async fn health_check_handler() -> Json<String> {
//     Json("liveness,sir".to_string())
// }

pub async fn health_check_handler() -> String {
    "liveness,sir".to_string()
}
