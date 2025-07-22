# SFX framework 

SFX is a simplified framework for building full-stack, small service. 

The framework now allows external auth, local auth and we will soon add OAuth into it 

The framework is built using starberry. Read more about starberry in https://fds.rs/starberry/. 

You may import starberry directly from `use sfx::prelude` or `use sfx::starberry` 

Use `sfx --help` to learn how to use built-in tools in sfx, while run `sfx init` in the target dir to initialize a new project 

https://fds.rs/sfx/tutorial/0.1.3/ 

# Settings & Op 

## Endpoints 

### `/op/lang/<lang>` 

Change the user's language by setting a cookie and redirecting to the same page 
This may not work if running in http but not https 

##### Request 
`GET /op/lang/<lang>` 
EMPTY 

##### Response 
A `HttpResponse` that redirects to the same page with the new language set in a cookie 

Serves the static files 

**`/static/<path>` 

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
            "iurl": "/starberry/news/"
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
**`GET /user/login`**  
Renders login page with language-specific content  
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

### Admin 

# APIs 
