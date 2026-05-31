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

const DEFAULT_ROBOTS: &str = "User-agent: *\nDisallow: /user/\nDisallow: /admin/\n";

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

/// Create a page property object for rendering, with explicit SEO `keywords`.
///
/// # Arguments
/// * `req`         - The request context
/// * `title`       - The title of the page
/// * `description` - The description of the page
/// * `keywords`    - Comma-separated `<meta name="keywords">` value
///
/// # Returns
/// A `Value` object containing the page properties
pub fn pageprop_with_keywords(
    req: &mut HttpReqCtx,
    title: &str,
    description: &str,
    keywords: &str,
) -> Value {
    let lang = lang(req);
    let user_value: Value = req.params.get::<User>().unwrap().clone().into();
    let path = req.path();
    object!({
        lang: &lang,
        title: title,
        color: "pink",
        description: description,
        keywords: keywords,
        nav: NAVBAR.get(&lang).clone(),
        foot: FOOTER.get(&lang).clone(),
        user: user_value,
        path: path,
    })
}

/// Create a page property object for rendering.
///
/// Convenience wrapper around `pageprop_with_keywords` that leaves the SEO
/// `keywords` field empty. Pages that want to populate
/// `<meta name="keywords">` should call `pageprop_with_keywords` directly.
///
/// # Arguments
/// * `req` - The request context
/// * `title` - The title of the page
/// * `description` - The description of the page
///
/// # Returns
/// A `Value` object containing the page properties
pub fn pageprop(req: &mut HttpReqCtx, title: &str, description: &str) -> Value {
    pageprop_with_keywords(req, title, description, "")
}

/// Render a 403 Forbidden HTML page inside the site chrome.
///
/// Returns an `HttpResponse` with status `403` whose body is the
/// `user/forbidden.html` template. The template receives:
/// - `pageprop` — standard page properties (navbar + footer + lang)
/// - `message` — caller-supplied explanation, or a generic default
/// - `next`    — the current request URL, percent-encoded, suitable for
///   building a `/user/login?next=<next>` recovery link
///
/// Callers should use this in place of `text_response("403 Forbidden")` so
/// that users who hit a stale permission-gated URL after their session
/// expires still get the navbar / "Log in" / "Back to view" affordances
/// instead of a bare text page.
pub fn forbidden_response(req: &mut HttpReqCtx, message: Option<&str>) -> HttpResponse {
    let current = req.request.meta.url();
    let next = hotaru_lib::url_encoding::encode_url_owned(&current);
    akari_render!(
        "user/forbidden.html",
        pageprop = pageprop(req, "Forbidden", ""),
        message = message.unwrap_or("You don't have permission to access this resource."),
        next = next,
    ).status(StatusCode::FORBIDDEN)
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
            return Ok(req)
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
            return Ok(req)
        } else {
            next(req).await
        }
    }
}

pub use crate::admin::RedirectNonAdmin; 

// !TODO! Optimize match, such as, 'zh-hant' when not supported use 'zh-xxx' or 'zh' first
/// Resolve the language for the current request.
///
/// Resolution order:
/// 1. `?lang=<code>` query parameter — used by crawlers and `<link
///    rel="alternate" hreflang>` so each language has its own crawlable URL.
/// 2. `lang` cookie — set by the footer language switcher for human users.
/// 3. `default_lang()` — site fallback.
///
/// A value is accepted only if it appears in `SUPPORT_LANG`; an unrecognized
/// value at any layer falls through to the next.
///
/// # Arguments
/// * `req` - The request context
pub fn lang(req: &mut HttpReqCtx) -> String {
    lang_or_none(req).unwrap_or_else(default_lang)
}

/// Like `lang(req)` but returns `None` when neither the query string nor a
/// cookie explicitly set a supported language. Lets downstream apps insert
/// their own fallback (e.g. a `/<code>` URL-prefix scheme) between SFX's
/// query/cookie layer and the site default — something `lang()` collapses
/// into the same "default_lang()" answer.
pub fn lang_or_none(req: &mut HttpReqCtx) -> Option<String> {
    if let Some(q) = req.query("lang") {
        if SUPPORT_LANG.contains(&q.clone().into()) {
            return Some(q);
        }
    }
    if let Some(c) = req.get_cookie("lang") {
        let v = c.get_value().to_string();
        if SUPPORT_LANG.contains(&v.clone().into()) {
            return Some(v);
        }
    }
    None
}

/// Resolve the "previous page" for redirects (used by the language switcher).
///
/// Reads the `Referer` request header and returns the path-and-query
/// portion, stripping `scheme://host` if present. Falls back to `"/"` when
/// the header is absent, empty, or shaped in a way we don't recognize.
/// Browsers send `Referer` on same-origin clicks by default, so the footer
/// language links no longer need an explicit `?from=…` query param.
///
/// # Remark
///
/// This depends on the client sending the `Referer` header. It may fail
/// (i.e., redirect to `/`) on browsers with referrer-policy set to
/// `no-referrer`, in privacy modes that strip referrers, or in
/// non-browser / non-standard clients. Same-origin defaults send it.
pub fn from(req: &mut HttpReqCtx) -> String {
    let referer = match req.header_str("referer") {
        Some(r) if !r.is_empty() => r.to_string(),
        _ => return "/".to_string(),
    };
    if let Some(scheme_end) = referer.find("://") {
        let after_scheme = &referer[scheme_end + 3..];
        match after_scheme.find('/') {
            Some(slash) => after_scheme[slash..].to_string(),
            None => "/".to_string(),
        }
    } else if referer.starts_with('/') {
        referer
    } else {
        "/".to_string()
    }
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

endpoint! {
    APP.url("/robots.txt"),

    /// Serve the site's `robots.txt`.
    ///
    /// Reads `programfiles/op/robots.txt` from the current working directory
    /// if present, otherwise falls back to a built-in default that disallows
    /// `/user/` and `/admin/`. Consumers can override the default by shipping
    /// their own file at that path.
    ///
    /// # Request
    /// `GET /robots.txt`
    ///
    /// # Returns
    /// A `text/plain` `HttpResponse` with the robots directives.
    pub robots_txt <HTTP> {
        let _ = req;
        let path = env::current_dir().unwrap_or_default()
            .join("programfiles/op/robots.txt");
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| DEFAULT_ROBOTS.to_string());
        text_response(body)
    }
}
