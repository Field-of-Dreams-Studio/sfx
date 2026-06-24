use hotaru::prelude::*;
use hotaru::http::*;
use htmstd::{CookieSession, PreferredLanguageMiddleware, PreferredLanguageSettings, PrintLog};

pub mod prelude {
    pub use hotaru::prelude::*;
    pub use hotaru::http::*;
    pub use htmstd::{
        CookieSession, Cors, PreferredLanguage, PreferredLanguageMiddleware,
        PreferredLanguageRequestExt, PreferredLanguageSettings, PrintLog, cors_settings,
    };
    pub use hotaru;
}

pub use hotaru;

pub mod op;
pub mod user;
pub mod local_auth;
pub mod admin;

pub static APP: SServer = Lazy::new(|| {
    Server::new()
        // .mode(RunMode::Build)
        .binding(op::BINDING.clone())
        .max_connection_time(TimeoutSetting::Seconds(10))
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default()))
            .append_middleware::<PrintLog>()
            .append_middleware::<CookieSession>()
            .append_middleware::<PreferredLanguageMiddleware>()
            .append_middleware::<user::UserFetch>()
        )
        .set_config(
            prelude::cors_settings::AppCorsSettings::new()
        )
        .set_config(
            PreferredLanguageSettings::new(op::default_lang())
        )
        .build()
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
