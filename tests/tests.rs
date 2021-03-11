use std::sync::Arc;
use async_tungstenite::{tokio::connect_async_with_tls_connector, tungstenite::Message as Message};
use anyhow::Result;
use tokio_rustls::{TlsConnector, rustls::ClientConfig};
use futures::{StreamExt, SinkExt};
use futures::prelude::*;
use jackrabbit::*;

#[tokio::test]
async fn test1() -> Result<()> {

    let mut config = ClientConfig::new();
    config.root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    let config = TlsConnector::from(Arc::new(config));

    let (ws_stream, _) = connect_async_with_tls_connector("wss://api.dispatcher.dev", Some(config)).await.unwrap();
    let (mut ws_write, ws_read) = ws_stream.split();

    let mut stream = ws_read.into_stream();

    let f_data = "hello".to_string();
    let i_data = rmp_serde::to_vec_named(&f_data).unwrap();
    let req: Req = Req{
        command: Command::Add,
        data: i_data.clone(),
    };
    let req = rmp_serde::to_vec_named(&req).unwrap();

    ws_write.send(Message::Binary(req)).await.unwrap();

    let req: Req = Req{
        command: Command::Take,
        data: Vec::new(),
    };
    let req = rmp_serde::to_vec_named(&req).unwrap();

    ws_write.send(Message::Binary(req)).await.unwrap();

    'outer: loop {
        #[allow(irrefutable_let_patterns)]
        while let res = stream.next().await {
            match res {
                Some(result) => {
                    match result {
                        Ok(message) => {
                            if message.is_binary() {
                                let data = message.into_data();
                                let res: SavedReq = rmp_serde::from_read_ref(&data).unwrap();
                                if res.command == Command::Take {
                                    println!("{:?}", res);
                                    assert_eq!(res.data, i_data);
                                    break 'outer
                                }
                            }
                        },
                        Err(e) => { println!("error 1: {}", e); break 'outer },
                    };
                },
                None => { break 'outer },
            }
        }
    }

    Ok(())
}