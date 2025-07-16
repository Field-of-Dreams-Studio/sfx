use starberry::prelude::*;
use sbmstd::{CookieSession, PrintLog}; 

pub mod prelude { 
    pub use starberry::prelude::*;
    pub use sbmstd::{CookieSession, PrintLog, Cors, cors_settings}; 
    pub use starberry; 
} 

pub use starberry; 

pub mod op; 
pub mod user; 
pub mod local_auth; 
pub mod admin; 

pub static APP: SApp = Lazy::new(|| {
    App::new()
        // .mode(RunMode::Build) 
        .binding(op::BINDING.clone())
        .max_connection_time(10) 
        .single_protocol(ProtocolBuilder::<HttpReqCtx>::new()
            .append_middleware::<PrintLog>() 
            .append_middleware::<CookieSession>() 
            .append_middleware::<user::UserFetch>() 
        ) 
        .set_config(
            prelude::cors_settings::AppCorsSettings::new() 
        ).build() 
}); 

// #[url(APP.lit_url("/"))]
// async fn home_route() -> HttpResponse { 
//     println!("{}", req.path()); 
//     akari_render!(
//         "index.html", 
//         pageprop = op::pageprop(req, "Home", "Welcome to the SchemOp Home Page"), 
//         path = op::into_path_l(req, vec!["home"]) 
//     ) 
// }  
