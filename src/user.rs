pub const CACHE_VALID_TIME: u64 = 60 * 60; // 1 hour  

pub const HALF_VALID_TIME: u64 = CACHE_VALID_TIME / 2; 

pub mod endpoints; 
pub mod fetch; 
pub mod user; 
pub mod middleware; 
pub mod server; 

pub use user::{User, UserID}; 
pub use middleware::UserFetch; 
pub use server::Server; 
