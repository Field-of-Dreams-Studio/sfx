use sfx::prelude::*;
pub use sfx::APP;
pub use sfx::op;

endpoint! {
    APP.url("/"),

    pub home_route <HTTP> {
        println!("{}", req.path());
        akari_render!(
            "index.html",
            pageprop = op::pageprop(req, "Home", "Welcome to the SFX Home Page"),
            path = op::into_path_l(req, vec!["home"])
        )
    }
}
