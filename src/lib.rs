use hotaru::prelude::*;
use hotaru::http::*;
use htmstd::{CookieSession, PrintLog}; 

pub mod prelude { 
    pub use hotaru::prelude::*;
    pub use hotaru::http::*;
    pub use htmstd::{CookieSession, PrintLog, Cors, cors_settings}; 
    pub use hotaru; 
} 

pub use hotaru; 

pub mod op; 
pub mod user; 
pub mod local_auth; 
pub mod admin; 

pub static APP: SApp = Lazy::new(|| {
    App::new()
        // .mode(RunMode::Build) 
        .binding(op::BINDING.clone())
        .max_connection_time(10) 
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default()))
            .append_middleware::<PrintLog>() 
            .append_middleware::<CookieSession>() 
            .append_middleware::<user::UserFetch>() 
        ) 
        .set_config(
            prelude::cors_settings::AppCorsSettings::new() 
        ).build() 
}); 

// endpoint! {
//     APP.url("/"),
//     pub home_route <HTTP> { 
//         println!("{}", req.path()); 
//         akari_render!(
//             "index.html", 
//             pageprop = op::pageprop(req, "Home", "Welcome to the SchemOp Home Page"), 
//             path = op::into_path_l(req, vec!["home"]) 
//         ) 
//     }
// }
