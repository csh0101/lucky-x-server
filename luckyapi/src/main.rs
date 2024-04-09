
// use crate::xerror;
use anyhow::{Ok, Result};
use tracing::instrument;
use std::sync::Arc;
use clap::{crate_authors,crate_description, Parser, Subcommand};
use luckyapi::{handlers::zip_handler::{FileBundle, self}, *};
use luckylib::oltp_config::setup_tracing;
use axum::{handler::HandlerWithoutStateExt, routing::{get, post}, Router,
    extract::Extension,
};
use tokio::runtime::Builder;
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
    Copy{
        #[arg(short,long)]
        from_dir :String,
        #[arg(short,long)]
        to_dir:String
    },
    Zip {
        #[arg(short,long)]
        from_dir :String,
        // #[arg(short,long)]
        // to_dir:String
    }
}


fn main() -> Result<()> {

    let runtime = Builder::new_multi_thread()
        .worker_threads(4) // 设置工作线程数量为4
        .max_blocking_threads(32) // 设置最大阻塞线程数为32
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        app_init().await
    })

}


async fn app_init() -> Result<()>{



    #[cfg(feature="async")]
    println!("async");

    #[cfg(not(feature="async"))]
    println!("not async");
    //初始化tracing的问题
    setup_tracing().expect("setup tracing error");
    // let m = Command::new("cmd").author(crate_authors!("\n")).get_matches();
    let args = Cli::parse();
    match args.cmd {
        Commands::LaunchXServer { port } => {
            test_instrument("xyz".to_string()).await;

            register_router(port).await
        }
        Commands::AdHoc => {

            let x = crate::handlers::zip_handler::build_zip(FileBundle
                { path:  "[{\"filepath\": \"/Users/csh0101/lab/rust-playground/lucky-x-server/pictures\"}]".to_string(),
                deltarget: Some(0), key: Some("123".to_string()), filename: "zip_test_file".to_string() }).unwrap();
            println!("output dir {}",x);
            Ok(())
        },
        Commands::Copy { from_dir, to_dir } =>  {
             let count  = crate::parallel_copy(&from_dir, &to_dir).await? ;
             println!("copy content count: {}",count);
             Ok(())
        }
        ,
        Commands::Zip { from_dir } => {
        if let Err(e)  =  async_build_zip(Arc::new(luckyapi::AppContext::init()), FileBundle { path: from_dir, key:Some( "123".to_string()),
                deltarget: Some(0),
                filename: "test_pictures".to_string(),
        }).await{
            println!("{}",e)  
        }
        Ok(())
        },
    }

}

#[instrument(fields(custom_field="测试应用"))]
async fn test_instrument(test_str : String)  {
    tracing::info!("{}",test_str.to_string())
}


#[instrument(fields(custom_field = "应用注册信息"))]
async fn register_router(port: String) -> Result<()> {


    tracing::info!("register router....");


    let app = Router::new()
    .route("/api/v1/health", get(luckyapi::health_check_handler))
    .route("/api/v1/zipfile_bundle", post(luckyapi::zipfile_bundle))
    .with_state(Arc::new(luckyapi::AppContext::init()));
    // server listener
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}",port)).await.unwrap();


    axum::serve(listener,app).await.unwrap();

    
    // let rt = runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    // rt.block_on(launch(port))
    Ok(())
}

#[allow(dead_code)]
async fn launch(port: String) -> Result<()> {
    println!("{}", port);
    tracing::info!("csh0101");
    tracing::debug!("csh0101");
    let x =  std::env::var("RUST_LOG").unwrap();
    tracing::info!("the rust_log is {}",x);
    Ok(())
}



