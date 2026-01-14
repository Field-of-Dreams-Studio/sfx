pub use hotaru::prelude::*; 
pub use hotaru::http::*; 
pub use std::env;  

use crate::user;
use crate::user::User;
pub use crate::APP; 

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

pub static BINDING: Lazy<String> = Lazy::new(|| {
    let mut path = env::current_dir().unwrap();
    path.push("programfiles/op/binding.txt");
    std::fs::read_to_string(path)
        .unwrap_or_else(|_| "localhost:3003".to_string())
});

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

/// Create a page property object for rendering 
/// 
/// # Arguments 
/// * `req` - The request context
/// * `title` - The title of the page
/// * `description` - The description of the page
/// 
/// # Returns
/// A `Value` object containing the page properties 
pub fn pageprop(req: &mut HttpReqCtx, title: &str, description: &str) -> Value {
    let lang = lang(req); 
    let user_value: Value = req.params.get::<User>().unwrap().clone().into(); 
    let path = req.path(); 
    object!({
        lang: &lang,
        title: title, 
        color: "pink", 
        description: description,
        keywords: "", //"Hotaru, Akari, Project-StarFall",
        nav: NAVBAR.get(&lang).clone(),
        foot: FOOTER.get(&lang).clone(), 
        user: user_value, 
        path: path, 
    })
}

/// Get the default language from the support languages list 
pub fn default_lang() -> String {
    SUPPORT_LANG.idx(0).string()
} 

/// Check if the host is trusted 
pub fn is_trusted(host: String) -> bool { 
    TRUSTED_ORIGIN
        .list()
        .iter()
        .any(|v| v.string() == host || v.string() == LOCALHOST) 
} 

/// Get the trusted host list
///
/// # Returns
/// A reference to the trusted host list as a `Value` 
pub fn get_host() -> &'static Value { 
    return &TRUSTED_ORIGIN 
} 

/// Get the default host from the trusted origin list 
/// 
/// # Returns
/// A `String` representing the default host
pub fn get_default_host() -> String { 
    return TRUSTED_ORIGIN.idx(0).string() 
} 

/// Get the admin list 
pub fn get_admin() -> &'static Value { 
    return &ADMINS 
} 

/// Convenience: pull the current `User` from `req.params` or fall back to `guest`.
pub async fn get_user(req: &mut HttpReqCtx) -> User { 
    user::fetch::get_user(req).await 
} 

/// Convenience: pull the current `User` from `req.params` or fall back to `guest`. 
/// And then convert into UserID 
pub async fn get_user_id(req: &mut HttpReqCtx) -> user::UserID { 
    user::fetch::get_user_id(req).await 
}

middleware! { 
    /// Middleware to redirect guest users to login page 
    /// **MUST ADD AFTER UserFetch MIDDLEWARE** 
    pub RedirectGuest <HTTP> { 
        let user = get_user_id(&mut req).await; 
        if user.is_guest() { 
            req.response = redirect_response("/user/login"); 
            return req 
        } else { 
            next(req).await 
        } 
    } 
} 

middleware! { 
    /// Middleware to send unauthorized json response for guest users 
    /// **MUST ADD AFTER UserFetch MIDDLEWARE** 
    pub UnauthGuest <HTTP> { 
        let user = get_user_id(&mut req).await; 
        if user.is_guest() { 
            req.response = json_response(object!({
                success: false,
                message: "Unauthorized"
            })); 
            return req 
        } else { 
            next(req).await 
        } 
    } 
}

pub use crate::admin::RedirectNonAdmin; 

// !TODO! Optimize match, such as, 'zh-hant' when not supported use 'zh-xxx' or 'zh' first 
/// Get the language from the request context 
/// 
/// # Arguments
/// * `req` - The request context
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

/// Get the 'from' URL argument from the request context 
pub fn from(req: &mut HttpReqCtx) -> String {
    let from = req.query("from");
    println!("From = {:?}", from); 
    from.unwrap_or_else(|| "/".to_string())
} 

/// A type alias for a path object 
/// Vector of tuples where each tuple contains a path segment name and its actual location url  
pub type Path = Vec<(String, String)>; 

/// Convert a request context into a path object 
/// 
/// # Arguments
/// * `req` - The request context
/// * `path` - A vector of path segment names 
/// 
/// # Returns
/// A `Value` object containing the path segments and their corresponding URLs 
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
            current_path = format!("{}/{}", current_path, req.segment(path_seg)); 
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

/// Get a localized string from the localization dictionary 
pub fn get_localized_string(key: &str, lang: &str) -> String {
    let dict = L10N.get(key); 
    match dict.try_get(lang) {
        Ok(value) => value.string(), 
        Err(hotaru::akari::ValueError::KeyNotFoundError) => {
            dict.get(default_lang()).string()
        }, 
        Err(_) => dict.string(), 
    } 
} 

endpoint! {
    APP.url("/op/lang/<lang>"),

    /// Change the user's language by setting a cookie and redirecting to the same page
    /// This may not work if running in http but not https
    ///
    /// # Request
    /// `GET /op/lang/<lang>`
    /// EMPTY
    ///
    /// # Response
    /// A `HttpResponse` that redirects to the same page with the new language set in a cookie
    pub change_language <HTTP> {
        let lang = req.param("lang").unwrap_or_else(default_lang);
        redirect_response(&from(req)).add_cookie(
            "lang",
            Cookie::new(lang)
                .path("/")
                .http_only(true) 
        )
    }
}

endpoint! {
    APP.url("/static/<**path>"),

    /// Serves the static files
    ///
    /// # Request
    /// `GET /static/<**path>`
    /// EMPTY
    ///
    /// # Returns
    /// A `HttpResponse` containing the static file or a 404 error if not found
    pub static_file <HTTP> {
        println!("templates{}", req.path());
        serve_static_file(&req.path()[1..])
    }
}

endpoint! {
    APP.url("/redirect"),

    /// Redirects to a given URL
    ///
    /// # Request
    /// `GET /redirect?url=<url>`
    /// EMPTY
    ///
    /// # Returns
    /// A `HttpResponse` that redirects to the specified URL
    pub redirect <HTTP> {
        let url = req.query("url").unwrap_or_else(|| "/".to_string());
        println!("Redirecting to: {}", url); 
        redirect_response(&url)
    }
}
