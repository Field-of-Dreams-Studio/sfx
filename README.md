# SFX framework 

SFX is a streamlined, full-stack Rust framework for building small web services with integrated authentication, localization, and config-driven UI components 

SFX is a simplified framework for building full-stack, small service. 

The framework now allows external auth, local auth and we will soon add OAuth into it 

The framework is built using hotaru. Read more about hotaru in https://fds.rs/hotaru/. 

You may import hotaru directly from `use sfx::prelude` or `use sfx::hotaru` 

Use `sfx --help` to learn how to use built-in tools in sfx, while run `sfx init` in the target dir to initialize a new project 

https://fds.rs/sfx/tutorial/0.1.3/ 

# Settings & Op 

## Endpoints 

### `/op/lang/<lang>`

Change the user's language by setting a cookie and redirecting back to the
page they came from (resolved via the `Referer` header — see
`op::from(req)`). This may not work if running in http but not https.

##### Request
`GET /op/lang/<lang>`
EMPTY

##### Response
A `HttpResponse` that redirects to the previous page with the new language set in a cookie

##### Language resolution

`op::lang(req)` (used by `pageprop`, `into_path_l`, etc.) resolves in this
order, accepting a value only if it appears in `support_lang.json`:

1. `?lang=<code>` query parameter — used by crawlers and
   `<link rel="alternate" hreflang>` so each language has its own crawlable URL.
2. `lang` cookie — set by the footer language switcher for human users.
3. `Accept-Language` header — negotiated via
   `htmstd::PreferredLanguage::best_match` against `support_lang.json`.
   Quality, header order, and supported-list order are honored per RFC 9110.
4. `default_lang()` — the first entry in `support_lang.json`.

`op::lang_or_none(req)` returns `None` at step 4 instead of the default, so
downstream apps can insert their own fallback (e.g. a `/<code>/...` URL
prefix scheme) between SFX's negotiation layer and the site default.

The `Accept-Language` layer requires `PreferredLanguageMiddleware` in the
protocol stack. SFX's bundled `APP` installs it automatically and configures
its fallback to match `default_lang()`. Downstream apps that build their own
`APP` should also append `PreferredLanguageMiddleware` (re-exported from
`sfx::prelude`) for the layer to take effect.

Handlers that want typed access to the parsed header can call
`req.params.get::<htmstd::PreferredLanguage>()` directly (or via the
`PreferredLanguageRequestExt` trait) — useful for non-template scenarios
like API content negotiation.

### `/static/<path>`

Serves the static files

##### Request
`GET /static/<path>`
EMPTY

##### Returns
A `HttpResponse` containing the static file or a 404 error if not found

### `/redirect?url=<url>`

Redirects to a given URL

##### Request
`GET /redirect?url=<url>`
EMPTY

##### Returns
A `HttpResponse` that redirects to the specified URL

### `/robots.txt`

Serves crawler directives. Reads `programfiles/op/robots.txt` from the
current working directory if present, otherwise returns a built-in default
that disallows `/user/` and `/admin/`. Consumers can override by shipping
their own file at that path.

##### Request
`GET /robots.txt`
EMPTY

##### Returns
A `text/plain` `HttpResponse` with the robots directives.

## Settings 

The framework loads critical configuration files at startup from programfiles/op/ and programfiles/admin_info/ directories. These include:

### UI Components 
navbar.json and footer.json define site-wide UI elements localized by language keys.

<details>

<summary><b>How to write the navbar.json</b></summary>  

Navbar.json is located at `./programfiles/op/navbar.json` 

The root is a dictionary. The keys are languages, while the values are the content of navbar for different languages 

```json 
{ 
    "language-key": {"content"}, 
    "en": {".."}, 
    "zh-tw": {".."} 
}
``` 

For the content, there must be 2 content, name and itemlist. Name will be shown in the Logo area while the item lists are the items shown on the navbar 

```json 
{ 
    "en": { 
        "name": "ICON", 
        "itemlist": [
            ["..."], 
            ["..."] 
        ] 
    }
}
``` 

For items in the item list, there are two kinds. The first kind is a single link item 

```json
{
    "display": "Single",
    "url": "/",
    "is_dropdown": false // Indicates here 
}, 
``` 

Another kind is dropdown, if you click the item the subitems contained in this item will be shown 

```json 
{
    "display": "List",
    "url": "/",             // Useless, but safer to keep it 
    "is_dropdown": true,    // Indicates it is a dropdown link 
    "dropdown": [           // Dropdown items 
        {
            "item": "Item 1",
            "iurl": "/"
        },
        {
            "item": "Item 2",
            "iurl": "/hotaru/news/"
        }
    ]
} 
``` 

</details> 

<br> 

<details> 

<summary><b>How to write the footer.json</b></summary>  

### Footer Configuration (`footer.json`)
Located at `./programfiles/op/footer.json`, this file defines localized footer content. The root is a dictionary where keys are language codes and values are footer configurations.

#### Structure
```json
{
  "language-key": {
    "items": [FooterColumn],
    "footer": "HTML string"
  }
}
```

#### Components
1. **Footer Column** (`items` array):
   - `name`: Column title (e.g., "Support", "Resources")
   - `itemlist`: Array of link objects:
     ```json
     {
       "display": "Link Text",
       "url": "/path-or-external-url"
     }
     ```

2. **Footer HTML** (`footer`):
   - Raw HTML string for copyright/legal text
   - Supports inline styling and anchor tags
   - Example: 
     ```json
     "footer": "&copy; 2025 <a href=\"/about\">Company</a>"
     ```

#### Full Example
```json
{
  "en": {
    "items": [
      {
        "name": "Support",
        "itemlist": [
          {"display": "Help Center", "url": "/support"},
          {"display": "Contact", "url": "/contact"}
        ]
      },
      {
        "name": "Language",
        "itemlist": [
          {"display": "English", "url": "/op/lang/en"},
          {"display": "日本語", "url": "/op/lang/ja"}
        ]
      }
    ],
    "footer": "&copy; 2025 MyApp | <a href='/privacy'>Privacy Policy</a>"
  }
}
```

</details> 

<br> 

### Localization 
l10n.json stores translated strings, support_lang.json lists supported languages (first entry is default). 

<details> 

<summary><b>How to write the l10n.json</b></summary>   

### Localization Configuration (`l10n.json`)
Located at `./programfiles/op/l10n.json`, this file provides localized string translations. The root is a dictionary where keys are phrase identifiers and values are language-specific translations.

#### Structure
```json
{
  "phrase_key": {
    "language-code": "translated_text",
    "en": "English text",
    "zh": "中文文本",
    "ja": "日本語テキスト"
  }
}
```

#### Example Usage
```rust
// In template rendering:
op::get_localized_string("login", &lang) // Returns "登录" for zh
```

#### Best Practices
1. Keep keys short and semantic (`"nav_home"` vs `"homepage_link"`)
2. Maintain consistent casing (lowercase recommended)
3. Add new languages to `support_lang.json` first
4. Escape special characters (`\n`, `\"`, `\\`)

```json
// Minimal complete example
{
  "welcome": {
    "en": "Welcome back!",
    "zh": "欢迎回来！",
    "ja": "おかえりなさい"
  },
  "error_404": {
    "en": "Page not found",
    "zh": "页面不存在",
    "ja": "ページが見つかりません"
  }
}
```

</details>

### Security 
hosts.json contains trusted origins (checked via is_trusted()), admins.json holds administrator data.

<details> 

<summary><b>How to write the hosts.json</b></summary>   

Hosts.json located at `./programfiles/op/hosts.json` 

It is a Json List. The order in the list will change the order different hosts present in Login, the 0th element will be the default host 

A special string `"local"` indecates we use local host 

Minimal example: 

```json 
["auth.fds.moe", "local"]
``` 

</details>

### Network 
binding.txt specifies server binding address (default: localhost:3003). 

Directly write your location in `programfiles/op/binding.txt` 

# User Login & Operations 

### User Endpoints 

### User Endpoints

#### 1. Login Flow
**`GET /user/login[?next=<path>]`**  
Renders login page with language-specific content.

The optional `?next=<path>` query carries the URL the user should land on
after a successful login. The shipped `login.html` JS picks a post-login
redirect target in this order:

1. `?next=<path>` on the login URL, if it is a same-origin path (starts
   with `/` and not `//`).
2. `document.referrer` if it is same-origin and not `/user/login` itself.
3. `/` as a safe fallback.

This is also the target for `op::forbidden_response`'s "Log in" link, so a
403 → login → original page round trip preserves the user's location.

*Response*: HTML page with:
- Localized navbar/footer
- Host selection dropdown (from `hosts.json`)
- Login form

**`POST /user/login`**  
Authenticates user credentials  
*Parameters* (URL-encoded form):  
- `host`: Authentication server ("local" for localhost)  
- `username`: User identifier  
- `password`: Plaintext password  

*Responses*:  
```json
// Success response is generated the backend. Please refer to the Local Auth 

// Failure
{
  "success": false,
  "message": "Invalid credentials"
}
```

*Sets Cookies*:  
- `access_token`: JWT for authenticated sessions  
- `host`: Base authentication server  

---

#### 2. Session Management
**`GET /user/logout`**  
Invalidates current session  
*Clears*: Access token cookie  
*Redirects*: To login page  

**`GET /user/refresh?redirect=<url>`**  
Refreshes access token  
*Redirects*: To specified URL after refresh  

**`GET /user/refresh_api`** (Testing)  
Returns new access token  
*Response*:  
```json
{ "access_token": "new.jwt.token" }
```

---

#### 3. User Information
**`GET /user/cached_info`**  
Returns cached user data from session  
*Response*:  
```json
{
  "uid": "user-id",
  "server": "auth.fds.moe",
  "username": "john_doe",
  "email": "user@example.com",
  "is_active": true,
  "is_verified": true,
  "cached_time": 1712345678
}
```

**`GET /user/info`** (Testing)  
Fetches fresh user info from auth server  
*Response*: Raw user object  

---

#### 4. Protected Routes
**`GET /user/home`**  
User dashboard (requires valid session)  
*Redirects*: To `/user/login` if unauthenticated  
*Renders*: `user/home.html` with:  
- Localized page title/description  
- Breadcrumb navigation  
- Current user object  

**`GET /user/unauthorized`**  
Access denied page  
*Renders*: `user/unauthorized.html`  

##### `op::pageprop` / `op::pageprop_with_keywords`

`pageprop(req, title, description)` builds the standard page properties
(`lang`, `title`, `description`, `nav`, `foot`, `user`, `path`, etc.) and
leaves `<meta name="keywords">` empty.

To populate per-page SEO keywords without rebuilding the dict, call
`pageprop_with_keywords` instead:

```rust
use sfx::op;

let pp = op::pageprop_with_keywords(
    req,
    "Constitution",
    "The constitution of the project",
    "hotaru, akari, project starfall",
);
```

The four-argument form is the canonical entry point; `pageprop` is the
zero-keywords convenience wrapper.

##### `op::forbidden_response(req, message)` helper

For permission-gated endpoints, prefer this helper over
`text_response("403 Forbidden")` so users stranded on a gated URL after
their session expires still see the site chrome.

```rust
use sfx::op;

if !user.can_edit() {
    return op::forbidden_response(req, Some("You need write permission to edit this resource."));
}
```

Returns an `HttpResponse` with status `403` whose body renders
`user/forbidden.html` with:
- `pageprop` — standard page properties (navbar + footer + lang)
- `message` — caller-supplied explanation, or a generic default if `None`
- `next` — the current request URL, percent-encoded, suitable for building
  a `/user/login?next=<next>` recovery link in the template

The shipped template provides "Log in" (→ `/user/login?next=<current>`)
and "Back to view" (→ the current path with query stripped) buttons.

---

#### 5. Password Management
**`POST /user/home/change_password`**  
Updates user password  
*Parameters* (URL-encoded form):  
- `old_password`: Current password  
- `new_password`: New password  
- `host`: Authentication server  

*Responses*:  
```json
// Success
{ "success": true }

// Failure
{
  "success": false,
  "message": "Current password invalid"
}
```

### User API 

This module provides core utilities for managing user sessions, authentication tokens, and interactions with the authentication server.

#### Key Functions

##### Token Management
- **`set_auth_token(req: &mut HttpReqCtx, token: &str)`**  
  Stores the JWT token in the session under `"auth_token"`.
- **`get_auth_token(req: &HttpReqCtx) -> Option<String>`**  
  Retrieves the JWT token from the session, if present.
- **`set_host(req: &mut HttpReqCtx, host: &str)`**  
  Stores the authentication server host (e.g., `"auth.fds.moe"` or `"local"`) in the session.
- **`get_host(req: &HttpReqCtx) -> Server`**  
  Retrieves the authentication server host from the session, falling back to `Server::Local` if missing.

##### User Data Fetching
- **`fetch_user_info(host: Server, auth: String) -> Option<User>`**  
  Fetches user details from `/users/me` endpoint.  
  *Response format*: 
  ```json
  {
    "user": {
      "uid": "user-id",
      "username": "john_doe",
      "email": "user@example.com",
      "is_active": true,
      "is_verified": true
    }
  }
  ```

##### Session Operations
- **`refresh_user_token(req: &mut HttpReqCtx) -> Value`**  
  Refreshes access token via `/auth/refresh`. Updates session token on success.  
  *Success response*: 
  ```json
  { "success": true, "access_token": "new.jwt.token" }
  ```
- **`cache_user_info(req: &mut HttpReqCtx, user: User)`**  
  Caches deserialized user object in session.
- **`logout(req: &mut HttpReqCtx) -> HttpResponse`**  
  Clears session tokens and redirects to login flow.
- **`disable_token(host: Server, token: String) -> Value`**  
  Invalidates token server-side via `/auth/logout`.

##### Utilities
- **`request_with_auth_token()`**  
  Adds `Authorization: Bearer <token>` header to requests.
- **`get_user()`**  
  Retrieves current user from request params (guest if unauthenticated).
- **`auth_server_health()`**  
  Checks `/health` endpoint (returns `true` for `{ "status": "ok" }`).

#### Security Features
1. **HTTP-only Cookies**  
   Tokens are stored with `http_only` flag to prevent XSS access.
2. **Automatic Token Refresh**  
   `/auth/refresh` maintains session validity without re-login.
3. **Server-side Invalidation**  
   Tokens are disabled on logout via auth server.
4. **Guest Fallback**  
   Unauthenticated requests return `User::guest()`.

#### Flow Example
TBD 

### Admin Endpoints

The admin surface is split by content type: `/admin/panel/*` returns HTML,
`/admin/users/*` and `/admin/admins/*` return JSON. All endpoints check
`check_is_admin` first (the current request's `UserID` must appear in
`programfiles/admin_info/admins.json`). HTML pages redirect non-admins to
`/user/unauthorized`; JSON endpoints return `401 Unauthorized`.

#### 1. Pages (HTML)

**`GET /admin/`**  
Admin dashboard landing page.  
*Renders*: `admin/index.html` with localized page chrome.

**`GET /admin/panel`**  
User-management list.  
*Renders*: `admin/panel.html`, which loads `/admin/users/json` client-side
and paginates 10 rows per page.  
*Columns*: UID, Username, Email, Active, Action.  
*Links*: per-row "Edit" → `/admin/panel/<uid>`; "Manage admins" →
`/admin/panel/admins`.

**`GET /admin/panel/<uid>`**  
Edit page for one user. Returns `404` (HTML) if the uid doesn't exist.  
*Renders*: `admin/user_edit.html` with three forms:
- Edit username / email / active → `POST /admin/users/<uid>`
- Reset password → `POST /admin/users/<uid>/password`
- Delete (`confirm()` first) → `POST /admin/users/<uid>/delete`

All three forms submit as `application/x-www-form-urlencoded` via JS and
display the JSON response inline.

**`GET /admin/panel/admins`**  
Admin-membership management page.  
*Renders*: `admin/admins.html`. Lists entries from `admins.json` and
supports add/remove via the JSON API.

---

#### 2. User Management API (JSON)

**`GET /admin/users`**  
List all locally-stored users.  
*Response*:
```json
{
  "success": true,
  "total": 12,
  "users": [
    {
      "uid": 1,
      "username": "Admin",
      "email": "admin@example.com",
      "is_active": true,
      "is_admin": true
    }
  ]
}
```
`is_admin` is computed per request from `admins.json`; it is not a stored
field on `UserStorage`.

**`POST /admin/users`**  
Create a new local user.  
*Parameters* (URL-encoded form):  
- `username`: New user's identifier  
- `email`: Email address  
- `password`: Plaintext password (hashed server-side before storage)  

*Responses*:
```json
// Success (201 Created)
{ "success": true, "username": "alice" }

// 400 Bad Request — format validation failed
{ "success": false, "message": "Username is not valid" }

// 409 Conflict — value owned by another user
{ "success": false, "message": "Email already exists" }
```
Maps `FopError` variants to: `400` (`UserNameNotValid` / `EmailNotValid` /
`PasswordMismatch`), `409` (`UserNameConflict` / `EmailConflict`), `429`
(`TooManyRequest`), `500` (anything else, logged via `tracing::error!`).

**`GET /admin/users/json`**  
Identical payload to `GET /admin/users`; kept as the panel JS's stable
endpoint name.

**`GET /admin/users/<uid>`**  
Single-user JSON.  
*Responses*:
```json
// 200 OK
{
  "success": true,
  "user": { "uid": 1, "username": "Admin", ... }
}

// 404 Not Found
{ "success": false, "message": "User not found" }
```

**`POST /admin/users/<uid>`**  
Edit an existing user. All fields optional — only present fields are
applied.  
*Parameters* (URL-encoded form, all optional):  
- `username`: New username  
- `email`: New email  
- `is_active`: `"true" | "on" | "1" | "yes"` → `true`, anything else →
  `false`  

*Same-uid uniqueness exception*: passing the user's existing username or
email is allowed (no-op for that field). A duplicate value owned by a
different uid returns `409`.

*Responses*:
```json
// 200 OK
{ "success": true }

// 404 Not Found / 400 Bad Request / 409 Conflict
{ "success": false, "message": "..." }
```

**`POST /admin/users/<uid>/password`**  
Reset the user's password.  
*Parameters* (URL-encoded form):  
- `new_password`: Plaintext password (hashed server-side)

*Responses*:
```json
// 200 OK
{ "success": true }

// 400 Bad Request (empty password) / 404 Not Found
{ "success": false, "message": "..." }
```

**`POST /admin/users/<uid>/delete`**  
Delete a user. Returns `404` if the uid doesn't exist.  
*Responses*:
```json
// 200 OK
{ "success": true }

// 404 Not Found
{ "success": false, "message": "User not found" }
```

---

#### 3. Admin Membership API (JSON)

**`GET /admin/admins/json`**  
List current admin entries.  
*Response*:
```json
{
  "success": true,
  "admins": ["1@local", "3@local"]
}
```

**`POST /admin/admins`**  
Add an admin entry.  
*Parameters* (URL-encoded form):  
- `uid`: Either `"3"` (defaults to `@local`) or `"3@local"` (fully
  qualified). Must reference an existing local user for the `@local`
  case.

*Responses*:
```json
// 200 OK — idempotent: re-adding an existing entry succeeds silently
{ "success": true, "entry": "3@local" }

// 400 Bad Request — malformed or unknown uid
{ "success": false, "message": "Local user not found" }
```

**`POST /admin/admins/<entry>/delete`**  
Remove an admin entry. `<entry>` must be URL-encoded (e.g. `1%40local`
for `1@local`).  
*Responses*:
```json
// 200 OK — also idempotent for missing entries
{ "success": true }

// 400 Bad Request
{ "success": false, "message": "Invalid admin entry" }
```

---

#### 4. Backend additions

##### `AuthManager` (in `src/local_auth/fop.rs`)

New uid-keyed admin methods. None take a caller token — gating happens at
the endpoint layer:

- **`admin_list_users() -> Vec<(u32, UserStorage)>`**  
  Enumerates all users with their uid; uid-sorted.
- **`admin_get_user(uid) -> Option<UserStorage>`**  
  Single lookup; `None` if missing.
- **`admin_edit_user(uid, new_username, new_email, new_is_active) -> Result<(), FopError>`**  
  Partial update with same-uid uniqueness exception. Holds
  `username_map`, `email_map`, `users` write locks atomically; format
  checks run before the lock acquisition, conflict checks after.
- **`admin_reset_password(uid, new_password) -> Result<(), FopError>`**  
  Empty password → `PasswordMismatch`.
- **`admin_delete_user(uid) -> Result<(), FopError>`**  
  Removes from `users`, `username_map`, and `email_map` atomically. Does
  *not* clean orphan `admins.json` entries (tracked in `TICKETS.md`).

##### `UserStorage` (in `src/local_auth/fop.rs`)

New field:
- **`pub is_active: bool`**  
  Defaults to `true` on load from disk for backward compatibility with
  pre-0.1.3 user files. Inactive users cannot authenticate via
  `/auth/login`, `/auth/refresh`, or `/users/me`.

##### `FopError` (in `src/local_auth/fop.rs`)

New variants:
- **`UserNameConflict`**, **`EmailConflict`** — value owned by another
  uid. Surfaced by the API as `409 Conflict`, distinct from the format-
  validation `400` produced by `UserNameNotValid` / `EmailNotValid`.

##### `op` module (in `src/op.rs`)

Admins are now mutable at runtime. `ADMINS` switched from
`Lazy<Value>` to `Lazy<RwLock<Value>>`, and `get_admin()` returns an
owned clone rather than `&'static Value`:

- **`get_admin() -> Value`**  
  Returns a clone of the current admin snapshot.
- **`read_admin_entries() -> Vec<String>`**  
  Convenience: the snapshot as `Vec<String>`.
- **`add_admin_entry(entry: &str) -> std::io::Result<()>`**  
  Idempotent. Persists to `programfiles/admin_info/admins.json` first,
  then updates the in-memory snapshot.
- **`remove_admin_entry(entry: &str) -> std::io::Result<()>`**  
  Same write-through ordering.

##### `RedirectNonAdmin` middleware

Unchanged. Insert after `UserFetch` in your protocol stack to gate any
custom admin route. Non-admin requests are redirected to
`/user/unauthorized`.

# APIs

