use sfx::prelude::*; 
pub use sfx::APP; 
pub use sfx::op; 

#[url(APP.lit_url("/"))]
async fn home_route() -> HttpResponse { 
    println!("{}", req.path()); 
    akari_render!(
        "index.html", 
        pageprop = op::pageprop(req, "Home", "Welcome to the SFX Home Page"), 
        path = op::into_path_l(req, vec!["home"]) 
    ) 
}  
