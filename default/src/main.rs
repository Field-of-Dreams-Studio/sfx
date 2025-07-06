use starberry::prelude::*; 
use {{project_name}}::APP; 

#[tokio::main]
async fn main() { 
    APP.clone().run().await; 
} 
