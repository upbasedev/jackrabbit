## JackRabbit

[![crates.io](https://meritbadge.herokuapp.com/jackrabbit)](https://crates.io/crates/jackrabbit)

### Purpose

The purpose of this service is to be a persistant work queue that is multi-producer and single consumer like AMQP but using Rust, MessagePack, RocksDB, and secure websockets. 

Currently, the only feature that is not implemented is NTP timestamps as I have to make `broker-ntp` be async.

See the `tests` (directory) file for more information on how to build a working client. 

As this is based on [MessagePack](https://msgpack.org/index.html) - JackRabbit supports over 50 languages.

This service is very lightweight as it uses almost no memory and CPU.

The current version does not delete any messages as it may in the future support replay.

### Install

``` cargo install jackrabbit ```

- the port needs to be passed in as a flag - default `443`
- the db (path) where you want the embedded rocksdb to be saved - default `tmp`
- the chain_cert (path) where the chain certificate is located - default `certs/chain.pem`
- the private_key (path) where the private key (certificate) is located - default `certs/private_key.pem`

- example: jackrabbit --save-path tmp --port 443 --cert certs/chain.pem --cert-path certs/private_key.pem
OR
- example: jackrabbit (using defaults)

### Service

There is an example `systemctl` service for Ubuntu called `jackrabbit.service` in the code
