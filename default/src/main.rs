use sfx::prelude::*;
use {{crate_name}}::APP;

#[tokio::main]
async fn main() {
    APP.clone().run().await;
}
