use std::sync::Arc;
use serde_derive::{Deserialize, Serialize};
use anyhow::Result;
use async_tungstenite::{tokio::accept_async, tungstenite::Message};
use tokio::net::TcpListener;
use futures::{StreamExt, SinkExt};
use futures::prelude::*;
use tokio_rustls::{TlsAcceptor, rustls::NoClientAuth, rustls::{ServerConfig, internal::pemfile}};
use tokio::fs::read;
use lazy_static::lazy_static;

lazy_static! {
    static ref DB : Arc<rocksdb::DB> = {

        let prefix_extractor = rocksdb::SliceTransform::create_fixed_prefix(3);

        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.set_prefix_extractor(prefix_extractor);

        let configure = env_var_config();
        let db = rocksdb::DB::open(&opts, configure.db).unwrap();
        Arc::new(db)
    };
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvVarConfig {
    pub port: usize,
    pub db: String,
    pub chain_cert: String,
    pub private_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub command: Command,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SavedReq {
    pub command: Command,
    pub id: uuid::Uuid,
    pub created_at: i64,
    pub published: bool,
    pub published_at: i64,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum Command {
    Add,
    Take,
}

fn replace(key: String, value: Vec<u8>) -> Result<()> {
    DB.put(key.clone(), value.clone())?;
    Ok(())
}

fn get_saved_reqs() -> Result<Vec<SavedReq>> {
    let prefix = "req".as_bytes();
    let i = DB.prefix_iterator(prefix);
    let res : Vec<SavedReq> = i.map(|(_, v)| {
        let data: SavedReq = rmp_serde::from_read_ref(&v).unwrap();
        data
    }).collect();
    Ok(res)
}

fn puts_saved_req(saved_req: SavedReq) -> Result<()> {
    let key = format!("req_{}", saved_req.id);
    let value = rmp_serde::to_vec_named(&saved_req)?;
    replace(key, value)?;
    Ok(())
}

// config based on sane local dev defaults (uses double dashes for flags)
fn env_var_config() -> EnvVarConfig {
 
    let mut db : String = "tmp".to_string();
    let mut port : usize = 443;
    let mut chain_cert : String = "certs/chain.pem".to_string();
    let mut private_key : String = "certs/private_key.pem".to_string();
    let _ : Vec<String> = go_flag::parse(|flags| {
        flags.add_flag("db", &mut db);
        flags.add_flag("port", &mut port);
        flags.add_flag("chain_cert", &mut chain_cert);
        flags.add_flag("private_key", &mut private_key);
    });

    EnvVarConfig{port, db, chain_cert, private_key}
}

pub async fn rabbit() {
    
    // create server
    let configure = env_var_config();
    let address = format!("0.0.0.0:{}", configure.port);
    let listener = TcpListener::bind(address).await.unwrap();

    // setup keys/certs
    let configure = env_var_config();
    let mut config = ServerConfig::new(Arc::new(NoClientAuth));
    let chain_cert = read(configure.chain_cert).await.unwrap();
    let cert_chain = pemfile::certs(&mut &chain_cert[..]).unwrap();
    let key = read(configure.private_key).await.unwrap();
    let key_der = pemfile::pkcs8_private_keys(&mut &key[..]).unwrap().first().unwrap().clone();
    config.set_single_cert(cert_chain, key_der).unwrap();

    #[allow(irrefutable_let_patterns)]
    while let conn = listener.accept().await {

        let config = config.clone();
        let acceptor = TlsAcceptor::from(Arc::new(config));

        // spawn new thread
        tokio::spawn(async move {

            let acceptor = acceptor.clone();

            let (tcp_stream, _) = conn.unwrap();

            let tls_stream = acceptor.accept(tcp_stream).await.unwrap();

            let ws_stream = accept_async(tls_stream).await.unwrap();

            let (mut ws_write, ws_read) = ws_stream.split();

            let mut stream = ws_read.into_stream();
    
            'outer: loop {

                #[allow(irrefutable_let_patterns)]
                while let res = stream.next().await {

                    match res {
                        Some(result) => {
                            match result {
                                Ok(message) => {
                                    if message.is_binary() {
                                        let req: Req = rmp_serde::from_read_ref(&message.into_data()).unwrap();
                                        if req.command == Command::Add {
                                            let id = uuid::Uuid::new_v4();
                                            let created_at = nippy::get_unix_ntp_time().await.unwrap();
                                            let published = false;
                                            let published_at = 0;
                                            let saved_req = SavedReq{id, created_at, published, published_at, command: Command::Take, data: req.data};
                                            puts_saved_req(saved_req.clone()).unwrap();
                                        }
                                        else if req.command == Command::Take {
                                            match get_saved_reqs() {
                                                Ok(saved_reqs) => {
                                                    for mut saved_req in saved_reqs {
                                                        if !saved_req.published {
                                                            let data = rmp_serde::to_vec_named(&saved_req).unwrap();
                                                            match ws_write.send(Message::Binary(data)).await {
                                                                Ok(_) => {
                                                                    saved_req.published_at = nippy::get_unix_ntp_time().await.unwrap();
                                                                    saved_req.published = true;
                                                                    puts_saved_req(saved_req).unwrap();
                                                                },
                                                                Err(_) => { break 'outer }
                                                            }
                                                        }
                                                    }
                                                },
                                                Err(_) => { break 'outer }
                                            }    
                                        }
                                    }
                                },
                                Err(_) => { break 'outer }
                            }
                        },
                        None => { break 'outer }
                    }
                }
            }
        });
    }
}
