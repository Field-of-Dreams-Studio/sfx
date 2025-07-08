use crate::user::User;
pub use crate::APP; 
pub use starberry::prelude::*; 
pub use std::env;  

static NAVBAR: Lazy<Value> = Lazy::new(|| {
    let mut path = env::current_dir().unwrap();
    path.push("programfiles/op/navbar.json");
    Value::from_jsonf(path.to_str().unwrap()).unwrap_or(Value::None)
});

static FOOTER: Lazy<Value> = Lazy::new(|| {
    let mut path = env::current_dir().unwrap();
    path.push("programfiles/op/footer.json");
    Value::from_jsonf(path.to_str().unwrap()).unwrap_or(Value::None) 
});

static SUPPORT_LANG: Lazy<Value> = Lazy::new(|| {
    let mut path = env::current_dir().unwrap();
    path.push("programfiles/op/support_lang.json");
    Value::from_jsonf(path.to_str().unwrap()).unwrap_or(Value::None)
});

static L10N: Lazy<Value> = Lazy::new(|| {
    let mut path = env::current_dir().unwrap();
    path.push("programfiles/op/l10n.json");
    Value::from_jsonf(path.to_str().unwrap()).unwrap_or(Value::None)
}); 

static ADMINS : Lazy<Value> = Lazy::new(|| {
    let mut path = env::current_dir().unwrap();
    path.push("programfiles/admin_info/admins.json");
    Value::from_jsonf(path.to_str().unwrap()).unwrap_or(Value::None)
}); 

static TRUSTED_ORIGIN : Lazy<Value> = Lazy::new(|| {
    let mut path = env::current_dir().unwrap();
    path.push("programfiles/op/hosts.json");
    Value::from_jsonf(path.to_str().unwrap()).unwrap_or(Value::None)
}); 

pub static BINDING: &str = "localhost:3003"; // Default binding address 

static LOCALHOST: &str = "local"; 

// fn create_static_value(
//     path: &str,
// ) -> Lazy<Value, Box<dyn Fn() -> Value + Send + Sync + 'static>> {
//     // clone the path into the closure
//     let p = path.to_string();
//     Lazy::new(Box::new(move || {
//         Value::from_jsonf(&p)
//             .map_err(|e| {
//                 println!("Failed to load static file: {} because of {}", p, e);
//                 e
//             })
//             .unwrap_or(Value::None)
//     }))
// }

pub fn pageprop(req: &mut HttpReqCtx, title: &str, description: &str) -> Value {
    let lang = lang(req); 
    let user_value: Value = req.params.get::<User>().unwrap().clone().into(); 
    let path = req.path(); 
    object!({
        lang: &lang,
        title: title, 
        color: "pink", 
        description: description,
        keywords: "", //"Starberry, Akari, Project-StarFall",
        nav: NAVBAR.get(&lang).clone(),
        foot: FOOTER.get(&lang).clone(), 
        user: user_value, 
        path: path, 
    })
}

pub fn default_lang() -> String {
    SUPPORT_LANG.idx(0).string()
} 

pub fn is_trusted(host: String) -> bool { 
    TRUSTED_ORIGIN
        .list()
        .iter()
        .any(|v| v.string() == host || v.string() == LOCALHOST) 
} 

pub fn get_host() -> &'static Value { 
    return &TRUSTED_ORIGIN 
} 

pub fn get_default_host() -> String { 
    return TRUSTED_ORIGIN.idx(0).string() 
} 

pub fn get_admin() -> &'static Value { 
    return &ADMINS 
} 

// !TODO! Optimize match, such as, 'zh-hant' when not supported use 'zh-xxx' or 'zh' first 
pub fn lang(req: &mut HttpReqCtx) -> String {
    let lang = req
        .get_cookie("lang")
        .map(|c| c.get_value().to_string())
        .unwrap_or_else(|| "".to_string());
    if SUPPORT_LANG.contains(&lang.clone().into()) {
        lang
    } else {
        default_lang()
    }
} 

pub fn from(req: &mut HttpReqCtx) -> String {
    println!("From = {:?}", req.get_url_args("from")); 
    req.get_url_args("from")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "/".to_string())
} 

pub type Path = Vec<(String, String)>; 

pub fn into_path(req: &mut HttpReqCtx, path: Vec<&str>) -> Value { 
    let mut value = object!([]); 
    let mut path_seg = 0; 
    let mut current_path = "".to_string(); 
    path.into_iter()
        .for_each(|name| { 
            value.push(object!({
                path: &current_path, 
                name: name 
            })); 
            current_path = format!("{}/{}", current_path, req.get_path(path_seg)); 
            path_seg += 1; 
        }); 
    let mut final_value = value.idx(value.len() - 1).clone(); 
    final_value.set("path", req.path()); 
    value.insert(value.len() - 1, final_value); 
    value 
} 

/// Use localized path for the path segments 
pub fn into_path_l(req: &mut HttpReqCtx, path: Vec<&str>) -> Value {
    let lang = lang(req);

    // 1) Collect owned Strings
    let localized: Vec<String> = path
        .into_iter()
        .map(|name| get_localized_string(name, &lang))
        .collect();

    // 2) Build a Vec<&str> that points into `localized`
    let slices: Vec<&str> = localized.iter().map(String::as_str).collect();

    // 3) Call into_path while `localized` (and thus `slices`) are still alive
    into_path(req, slices)
}

pub fn get_localized_string(key: &str, lang: &str) -> String {
    let dict = L10N.get(key); 
    match dict.try_get(lang) {
        Ok(value) => value.string(), 
        Err(starberry::akari::ValueError::KeyNotFoundError) => {
            dict.get(default_lang()).string()
        }, 
        Err(_) => dict.string(), 
    } 
} 

#[url(reg![&APP, LitUrl("op"), LitUrl("lang"), ArgUrl("lang")])]
async fn change_language() -> HttpResponse {
    // println!("Changing language to: {}", req.get_arg("lang").unwrap_or(default_lang()));
    redirect_response(
        &from(req) 
    ).add_cookie(
        "lang",
        Cookie::new(req.get_arg("lang").unwrap_or(default_lang()))
            .path("/")
            .http_only(true) 
    )
}

#[url(reg![&APP, LitUrl("static"), AnyPath()])]
async fn static_file() -> HttpResponse {
    println!("templates{}", req.path());
    serve_static_file(&req.path()[1..])
}

#[url(reg![&APP, LitUrl("redirect")])] 
async fn redirect() -> HttpResponse {
    let url = req.get_url_args("url").unwrap_or("/".to_string());
    println!("Redirecting to: {}", url); 
    redirect_response(&url)
} 
