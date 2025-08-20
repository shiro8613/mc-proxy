use std::{error::Error, fs::exists, process::exit, sync::Arc, time::Duration};

use tokio::{net::{TcpListener, TcpStream}, signal::ctrl_c, time::timeout};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{fmt::format::FmtSpan, fmt::time::LocalTime};

use crate::{config::Config, proxy::Proxy};


mod config;
mod proxy;
mod counter;
mod checker;
mod proxy_protocol;

struct App {
    config: Arc<Config>,
}

impl App {
    pub fn new(config :Config) -> Self {
        Self { 
            config: Arc::new(config),
        }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let config = self.config.clone();
        tracing::info!("Listen on {}", config.bind.as_str());
        let listener = TcpListener::bind(config.bind.as_str()).await?;

        let token = CancellationToken::new();
        let t = token.clone();

        tokio::spawn(async move {
            let _ = ctrl_c().await;
            tracing::info!("Shutdown...");
            t.cancel();
        });

        loop {
            tokio::select! {
                listener_await = listener.accept() => {
                    let (inbound, client_ip) = match listener_await {
                        Ok(a) => a,
                        Err(e) => {
                            tracing::error!("{}", e);
                            break;
                        }  
                    };
                        
                    tracing::info!("NewConnection: {}", client_ip.clone());
                    let outbound = TcpStream::connect(config.server.as_str());
                    let outbound = timeout(Duration::from_secs(config.timeout as u64), outbound).await??;
                    
                    let c_config = self.config.clone();
                    let c_token = token.clone();

                    tokio::spawn(async move {
                        let mut proxy = Proxy::new(c_config, c_token);
                        proxy.go(inbound, outbound, client_ip).await;
                    });
                }

                _ = token.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_timer(LocalTime::rfc_3339())
        .with_span_events(FmtSpan::FULL)
        .init();

    let path = "config.yml";
    let config = if exists(path).unwrap() {
        match Config::load(path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("{}", e);
                exit(1);
            }
        }
    } else {
        tracing::info!("Config file does not exits.");
        tracing::info!("Creating new config file");
        if let Err(e) = Config::save(path) {
            tracing::error!("{}", e);
            exit(1);
        }
        exit(1)
    };
    
    let app = App::new(config);
    if let Err(e) = app.run().await {
        tracing::error!("{}", e);
        exit(1);
    }
}
