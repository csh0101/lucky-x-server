
// use crate::xerror;
use anyhow::{Ok, Result};
use clap::{crate_authors,crate_description, Parser, Subcommand};
use luckyapi::{handlers::zip_handler::FileBundle, *};
use luckylib::tracing_config::setup_tracing;
use axum::{routing::{get, post}, Router};

// use lucky_x_api::error::storage_error::StorageError;



#[derive(Parser, Debug)]
#[command(
    version=build::VERSION, 
    about=crate_description!(), 
    subcommand_required=true,
    arg_required_else_help=true,
    long_about = None , 
    author=crate_authors!()
)]
struct Cli {
    #[clap(subcommand)]
    cmd: Commands,
    // 标记树？
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    LaunchXServer {
        #[arg(short, long, default_value = "8080")]
        port: String,
    },
    AdHoc,
}

#[tokio::main]
async fn main() -> Result<()> {

    //初始化tracing的问题
    setup_tracing();
    // let m = Command::new("cmd").author(crate_authors!("\n")).get_matches();
    let args = Cli::parse();
    match args.cmd {
        Commands::LaunchXServer { port } => register_router(port).await,
        Commands::AdHoc => {

            let x = crate::handlers::zip_handler::build_zip(FileBundle
                { path:"[\"/home/csh0101/lab/lucky-x-server/pictures\"]".to_string(), 
                deltarget: Some(0), key: Some("123".to_string()), filename: "zip_test_file".to_string() }).unwrap();
            println!("output dir {}",x);
            Ok(())
        },
    }
}

async fn register_router(port: String) -> Result<()> {


    tracing::info!("register router....");

    let app = Router::new()
    .route("/api/v1/health", get(luckyapi::health_check_handler))
    .route("/api/v1/zipfile_bundle", post(luckyapi::zipfile_bundle))
    ;

    // server listener
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}",port)).await.unwrap();


    axum::serve(listener,app).await.unwrap();

    
    // let rt = runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    // rt.block_on(launch(port))
    Ok(())
}

#[warn(dead_code)]
async fn launch(port: String) -> Result<()> {
    println!("{}", port);
    tracing::info!("csh0101");
    tracing::debug!("csh0101");
    let x =  std::env::var("RUST_LOG").unwrap();
    tracing::info!("the rust_log is {}",x);
    Ok(())
}

