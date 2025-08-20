use std::{net::SocketAddr, sync::Arc, time::Instant};
use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use tokio_util::sync::CancellationToken;

use crate::{checker::{Checker, CheckerState}, config::Config, counter::Counter, proxy_protocol::create_proxy_header};


pub struct Proxy {
    config :Arc<Config>,
    token :CancellationToken,
    client_ip :Option<SocketAddr>
}


impl Proxy {
    pub fn new(config :Arc<Config>, token :CancellationToken) -> Self {
        Self { 
            config,
            token,
            client_ip: None
        }
    }

    pub async fn go(&mut self, inbound :TcpStream, outbound :TcpStream, client_ip :SocketAddr) {
        self.client_ip = Some(client_ip);
        match self.handle(inbound, outbound).await {
            Ok(_) => {
                tracing::info!("[{}] Disconnected.", client_ip)
            },
            Err(e) => {
                tracing::warn!("ProxyThreadError: {}", e);
            }
        };
    }

    async fn handle(&mut self, inbound :TcpStream, outbound :TcpStream) -> Result<(), io::Error> {

        let (mut inbound_reader, mut inbound_writer) = inbound.into_split();
        let (mut outbound_reader, mut outbound_writer) = outbound.into_split();
        
        if self.config.proxy_protocol_v2 {
            if let Some(client_addr) = self.client_ip {
                if let Ok(server_addr) = self.config.server.parse() {
                    let header = create_proxy_header(client_addr, server_addr);
                    tracing::info!("{:?}", header);
                    let _ = outbound_writer.write_all(&header).await;
                } 
            }
        }
        
        let c_token = self.token.clone();
        let c_token_1 = self.token.clone();
        let c_token_2 = self.token.clone();
        let c_config = self.config.clone();

        let client_ip = if let Some(c) = self.client_ip {
                        c.to_string()
                    } else {
                        "nil".to_string()
                    };

        tokio::spawn(async move {
            let mut buf = vec![0; 8192];
            let mut counter = Counter::new(c_config.packet_per_sec);
            let mut checker = Checker::new(c_token_2);
            let mut check = false;

            let instant = Instant::now();
            checker.state_counter();
            while !c_token.is_cancelled() {
                if (c_config.waiting_minecraft_packet < instant.elapsed().as_secs()) && !check {
                    match checker.check() {
                        CheckerState::Ok => {
                            check = true;
                            tracing::info!("[{}] Validate Successfull.", client_ip)
                        },
                        CheckerState::Fail => {
                            tracing::warn!("[{}] Validate Failed. Dissconnect Client.", client_ip);
                            break;
                        }
                    }
                }

                match inbound_reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if counter.limit() {
                            tracing::warn!("[{}] packet is over max packet_per_sec. Disconnect Client", client_ip);
                            break;
                        }

                        if let Err(e) = outbound_writer.write_all(&buf[..n]).await { 
                            tracing::error!("{}", e);
                            break;
                        }

                        if n != 0 && !check {
                            checker.add_packet(buf[..n].to_vec());
                        }
                    },
                    Err(e) => {                        
                        tracing::error!("{}", e);
                        break;
                    }
                }
            }
            checker.stop();

        });

        tokio::select! {
            res = io::copy(&mut outbound_reader, &mut inbound_writer) => {
                res.map(|_| ())?
            }
            _ = c_token_1.cancelled() => {
                return Ok(());
            }
        }

        Ok(())
    }
}