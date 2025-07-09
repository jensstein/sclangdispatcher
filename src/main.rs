use std::io::{BufRead, Write};
use std::net::SocketAddr;

use clap::{Parser, Subcommand};
use warp::Filter;

type SCResult = anyhow::Result<()>;

/// Server and client for communicating with sclang
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start sclang server
    Server {
        /// Host address
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        /// Port for listening
        #[arg(short, long, default_value_t = 5000)]
        port: u16,
        /// Name of ide
        #[arg(short, long, default_value = "scvim")]
        ide_class: String,
    },
    /// Send command to the sclang server
    Client {
        /// Host address of server
        #[arg(long, default_value = "http://127.0.0.1")]
        host: String,
        /// Port of server
        #[arg(short, long, default_value_t = 5000)]
        port: u16,
        /// Command to send
        command: String,
    },
}

fn start_sclang(command_rx: std::sync::mpsc::Receiver<String>,
        ide_class: &str) -> SCResult {
    let mut output = std::process::Command::new("sclang")
        .args(["-i", ide_class])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    std::thread::spawn(move || {
        match output.stdout.as_mut() {
            Some(stdout) => {
                let mut out_reader = std::io::BufReader::new(stdout);
                loop {
                    let mut buf = String::new();
                    match out_reader.read_line(&mut buf) {
                        Ok(_) => {
                            if !buf.is_empty() {
                                log::info!("{buf}");
                            }
                        },
                        Err(error) => {
                            log::error!("Error: {error:?}");
                        }
                    }
                }
            },
            None => {
                log::error!("Unable to get stdout of sclang process");
                std::process::exit(1);
            }
        }
    });
    std::thread::spawn(move || {
        match output.stdin.as_mut() {
            Some(stdin) => {
                loop {
                    if let Err(error) = send_command(&command_rx, stdin) {
                        log::error!("Unable to send command to sclang \
                            process: {error}");
                        std::process::exit(1);
                    }
                }
            },
            None => {
                log::error!("Unable to get stdin of sclang process");
                std::process::exit(1);
            }
        }
    });
    Ok(())
}

fn send_command(command_rx: &std::sync::mpsc::Receiver<String>,
        stdin: &mut std::process::ChildStdin) -> SCResult {
    let received = command_rx.recv()?;
    stdin.write_all(format!("{received}").as_bytes())?;
    Ok(())
}

fn parse_command(command: bytes::Bytes, command_tx:
        &std::sync::mpsc::Sender<std::string::String>) -> SCResult {
    let command = String::from_utf8(command.to_vec())?;
    command_tx.send(command.to_string())?;
    Ok(())
}

#[derive(Debug)]
struct InvalidCommand;
impl warp::reject::Reject for InvalidCommand {}

async fn f(body: bytes::Bytes,
        command_tx: std::sync::mpsc::Sender<std::string::String>) ->
        Result<impl warp::Reply, warp::Rejection> {
    if let Err(error) = parse_command(body, &command_tx) {
        log::error!("Error happened when parsing command: {error}");
        return Err(warp::reject::custom(InvalidCommand))
    }
    Ok("")
}

pub fn start_server(addr: SocketAddr, command_tx:
        std::sync::mpsc::Sender<String>) ->
        impl std::future::Future<Output = ()> + 'static {
    let command_tx = warp::any().map(move|| command_tx.clone());
    let routes = warp::post()
        .and(warp::body::bytes())
        .and(command_tx)
        .and_then(f)
        .with(warp::log("sclangdispatcher"));
    warp::serve(routes).bind(addr)
}

#[tokio::main]
async fn main() -> SCResult {
    let args = Args::parse();
    env_logger::builder()
        .format(|buf, record| {
            let message = serde_json::json!({
                "level": record.level().to_string(),
                "message": record.args().as_str().map_or_else(||
                    {record.args().to_string()}, |s| s.to_string()),
                "target": record.target().to_string(),
            });
            writeln!(buf, "{}", message)
        })
        .filter(None, log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .init();
    match &args.command {
        Commands::Server{host, port, ide_class} => {
            let addr = format!("{}:{}", host, port);
            let (command_tx, command_rx) = std::sync::mpsc::channel();
            start_sclang(command_rx, ide_class)?;
            start_server(addr.parse()?, command_tx).await;
        },
        Commands::Client{command, host, port} => {
            let client = reqwest::Client::new();
            client.post(format!("{host}:{port}")).body(command.to_string())
                .send().await?;
        },
    }
    Ok(())
}
