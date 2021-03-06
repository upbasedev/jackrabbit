## JackRabbit

[![crates.io](https://meritbadge.herokuapp.com/jackrabbit)](https://crates.io/crates/jackrabbit)

The service that should always be running.

### Purpose

The purpose of this service is to be a real-time persistant message queue that is multi-producer and single consumer using Rust, MessagePack, RocksDB, and secure websockets.

This service is built to have multiple clients add messages while the main system processes the messages and a failover system is connected. The failover system is not getting messages and was connected second (the conneciton order matters).

This is useful for when you have one monolithic websocket system as an API and it needs to be updated (taken offline) - you can have the failover do the work while updating the main system. 

Then you take down the failover and the main system becomes the main system again.

See the `tests` (directory) file for more information on how to build a working client. 

As JackRabbit is based on [MessagePack](https://msgpack.org/index.html) so over 50 languages are supported.

This service is very lightweight as it uses almost no memory and CPU.

The current version does not delete any messages as it may in the future support replay.

JackRabbit requires an SSL cert and [LetsEncrypt](https://letsencrypt.org/) is recommended and also requires a DNS A record as `rustls` does not support IPs.

### Install

``` cargo install jackrabbit ```

- the port needs to be passed in as a flag - default `443`
- the db (path) where you want the embedded rocksdb to be saved - default `tmp`
- the chain_cert (path) where the chain certificate is located - default `certs/chain.pem`
- the private_key (path) where the private key (certificate) is located - default `certs/private_key.pem`

- example: jackrabbit --save-path="tmp" --port="443" --cert="certs/chain.pem" --cert-path="certs/private_key.pem"
- example: jackrabbit (using defaults)

### Service

There is an example `systemctl` service for Ubuntu called `jackrabbit.service` in the code

