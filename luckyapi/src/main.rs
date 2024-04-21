
// use crate::xerror;
use anyhow::{Ok, Result};
use etherparse::{err::{ip_exts, ipv4}, NetSlice, TcpSlice};
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
    },
    Tun {
        #[arg(short,long,default_value = "tun0")]   
        name  : String,
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
        Commands::Tun { name } => {
            println!("{}",name);
            loop {
            let nic = tun_tap::Iface::new(&name, tun_tap::Mode::Tun)?;

            let mut buf = [0u8;1504];

            let nbytes = nic.recv(&mut buf[..])?;


            let flags = u16::from_be_bytes([buf[0],buf[1]]);

            let proto = u16::from_be_bytes([buf[2],buf[3]]);

            if proto != 2048{
                println!("continue {}",proto);
                continue;
            }

            let ipv4_packet = etherparse::SlicedPacket::from_ip(&buf[4..]).unwrap();

            println!("read bytes {}, flag:{}, proto: {},{:?}",nbytes-4,flags,proto,&buf[4..nbytes]);

            println!("ipv4_packet: {:?}", ipv4_packet.net);

            if let Some(net_v4) = ipv4_packet.net {

                if let NetSlice::Ipv4(v4) = net_v4 {

                    let v4_header  = v4.header();

                    println!("version: {}\n
                    ihl : {}\n 
                    dcp: {}\n
                    ecn: {}\n
                    total_len:{}\na
                    Identification:{}\n
                    TTL:{}\n
                    Protocol:{:?}\n
                    Header Checksum:{}\n
                    Source Address:{}\n
                    Destination Address:{}\n
                    Options:{:?}\n
                    ",
                    v4_header.version(),
                    v4_header.ihl(),
                    v4_header.dcp(),
                    v4_header.ecn(),
                    v4_header.total_len(),
                    v4_header.identification(),
                    v4_header.ttl(),
                    v4_header.protocol(),
                    v4_header.header_checksum(),
                    v4_header.source_addr(),
                    v4_header.destination_addr(),
                    v4_header.options()
                );

                println!("payload_ip_number: {},",v4.payload_ip_number().protocol_str().unwrap());

               let  v4_payload = v4.payload();

               match v4_payload.len_source {
                    etherparse::LenSource::Slice => {
                        println!("other?")

                    },
                    etherparse::LenSource::Ipv4HeaderTotalLen => {
                        println!("ipv4")

                    },
                    etherparse::LenSource::Ipv6HeaderPayloadLen => {
                        println!("ip")

                    },
                    etherparse::LenSource::UdpHeaderLen => {
                        println!("upheader len")

                    },
                    etherparse::LenSource::TcpHeaderLen => {
                        println!("you should get payload length from tcp")
                    },
                }


                if let core::result::Result::Ok(tcp_slice) = etherparse::TcpSlice::from_slice(v4.payload().payload) {

                    let header =  tcp_slice.to_header();

                    println!("header: {:?}",header);

                    println!("header 
                    source port: {}
                    destination port:{}
                    sequence number: {}
                    acknowledgment number:{}
                    do: {}
                    window_size:{}
                    checksum: {}
                    urgent pointer: {}
                    options : {:?}
                    ",
                    header.source_port,
                    header.destination_port,
                    header.sequence_number,
                    header.acknowledgment_number,
                    header.data_offset(),
                    header.window_size,
                    header.checksum,
                    header.urgent_pointer,
                    header.options, 
                );
          println!("flag: 
                ns: {}
                fin : {}
                syn:{}
                rst : {}
                psh: {}
                ack: {}
                urg :{}
                ece: {}
                    ", header.ns,
                    header.fin,
                    header.syn,
                    header.rst,
                    header.psh,
                    header.ack,
                    header.urg,
                    header.ece,
                );


                

                


//    pub ns: bool,
//     /// No more data from sender
//     pub fin: bool,
//     /// Synchronize sequence numbers
//     pub syn: bool,
//     /// Reset the connection
//     pub rst: bool,
//     /// Push Function
//     pub psh: bool,
//     /// Acknowledgment field significant
//     pub ack: bool,
//     /// Urgent Pointer field significant
//     pub urg: bool,
//     /// ECN-Echo (RFC 3168)
//     pub ece: bool,
//     /// Congestion Window Reduced (CWR) flag
      

                let x =  header.options.elements_iter().for_each(|ele|{
                    let ele = ele.unwrap();
                    match ele {
                        // 无选项,用于分隔TCP选项字段中使用的不同选项。nop字段的实现取决于使用的操作系统。 
                        etherparse::TcpOptionElement::Noop => {
                            println!("this is no operation");
                        },
                        // 最大段大小  MSS 576 - 40 字节 标准的长度 = 536 
                        etherparse::TcpOptionElement::MaximumSegmentSize(s) => {
                            println!("mss: {}",s);
                        },
                        // 窗口缩放
                        etherparse::TcpOptionElement::WindowScale(scale) => {
                            println!("scale: {}",scale);
                        },
                        // 选择性确认批准
                        etherparse::TcpOptionElement::SelectiveAcknowledgementPermitted => {
                            println!("sack")
                        },
                        // 选择性确认
                        etherparse::TcpOptionElement::SelectiveAcknowledgement(x, y) => {
                            println!("sack :{}, {}",x.0,x.1);

                        },
                        // 时间戳 虚拟电路所尽力的往返传送时间，往返传输时间蒋准确确定TCP在尝试重新传输尚未确认的数据段之后
                        // 将等待多长时间
                        // Timestamp Echo 和 Timestamp Reply 字段相成。
                        etherparse::TcpOptionElement::Timestamp(x, y) => {
                            println!("timestamp:{},{}",x,y);
                        },
                    }
                });

                // xyz






                }







                // println!()

            //    println!("payload len_source:{}", v4_payload.len_source.);


            //    v4_payload.len_source
                

                    // v4_header.version()

                }

            }
            }

        }
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
        if let Err(e)  =  async_build_zip(Arc::new(luckyapi::AppContext::init()), 
        from_dir,"1.zip".to_string()).await{
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
    .route("/api/v1/process/:process_id", get(luckyapi::archive_procecss_status))
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



