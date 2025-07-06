pub mod fop; 
pub mod endpoints; 
pub mod analyze; 

use std::time::Duration;

use once_cell::sync::Lazy;

pub static LOCAL_AUTH: Lazy<fop::AuthManager> =
    Lazy::new(|| fop::AuthManager::new("programfiles/local_auth/users", Duration::from_secs(180))); 
