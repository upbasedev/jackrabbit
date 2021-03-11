mod lib;
use lib::rabbit;

#[tokio::main]
pub async fn main() {
    rabbit().await
}