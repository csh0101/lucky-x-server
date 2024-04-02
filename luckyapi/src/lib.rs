// 这种写法已经不在需要了
// #[macro_use]
// extern crate shadow_rs;
use shadow_rs::shadow;
shadow!(build);
pub mod common;
pub mod error;
pub mod handlers;

pub mod util;
pub use handlers::health_handler::health_check_handler;
pub use handlers::zip_handler::zipfile_bundle;
pub use util::parallel_copy;
