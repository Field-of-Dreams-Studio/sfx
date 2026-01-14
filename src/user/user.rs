//! user.rs
//!
//! Definition of the application’s `User` and `UserID` types, along with
//! (de)serialization to/from `hotaru::Value` and helper methods.

use hotaru::{object, Value}; 
use super::Server; 

/// Represents an authenticated user with metadata and a timestamp
/// for when the data was cached locally.
#[derive(Debug, Clone)]
pub struct User {
    /// Unique identifier and server origin
    pub id: UserID,
    username: String,
    email: String,
    is_active: bool,
    is_verified: bool,

    /// Instant at which this struct was created or last updated
    cached_at: u64,
}

impl User {
    /// Construct a new `User` with the given fields and set `cached_at` to now.
    pub fn new(
        id: UserID,
        username: String,
        email: String,
        is_active: bool,
        is_verified: bool,
    ) -> Self {
        Self {
            id,
            username,
            email,
            is_active,
            is_verified,

            cached_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Return the numeric user ID.
    pub fn get_uid(&self) -> usize {
        self.id.uid
    } 

    /// Return the server instance where the user account is stored. 
    pub fn get_server(&self) -> &Server {
        &self.id.server
    } 

    /// Return the full user ID struct. 
    pub fn get_user_id(&self) -> &UserID {
        &self.id
    } 

    /// Return the login/username.
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Return the email address.
    pub fn get_email(&self) -> &str {
        &self.email
    }

    /// `true` if the user account is active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// `true` if the user has verified their email.
    pub fn is_verified(&self) -> bool {
        self.is_verified
    }

    /// Compute time elapsed since `cached_at`.
    pub fn cache_age(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            - self.cached_at 
    } 

    /// Manually adjust the `cached_at` clock backward by `time` seconds,
    /// to simulate an older cache entry.
    ///
    /// # Arguments
    ///
    /// * `time` – optional number of seconds to rewind. `None` → no change.
    pub fn set_cached_time(mut self, time: Option<u64>) -> Self {
        if let Some(time) = time {
            self.cached_at = time
        };
        self
    } 

    /// Create an anonymous ("guest") user with ID 0 and default fields.
    pub fn guest(server: impl Into<Server>) -> Self {
        Self::new(
            UserID::new(0, server.into()),
            "Guest".into(),
            "guest@example.com".into(),
            false,
            false,
        )
    }
}

/// Construct a `User` from a `hotaru::Value` JSON object. Expects
/// fields `uid`, `username`, `email`, `is_active`, `is_verified` and
/// optionally `cached_time` (seconds old).
impl From<Value> for User {
    fn from(value: Value) -> Self {
        let base = User::new(
            UserID::new(value.get("uid").integer() as usize, value.get("server").string().into()),
            value.get("username").string(),
            value.get("email").string(),
            value.get("is_active").boolean(),
            value.get("is_verified").boolean(),
        );
        // rewind cache if provided
        let with_time = value
            .try_get("cached_time")
            .ok()
            .map(|v| v.integer() as u64);
        base.set_cached_time(with_time)
    }
}

/// Convert a `User` into a `hotaru::Value` map for JSON responses
/// or session storage. Fields:
/// - `uid`, `server`, `username`, `email`, `is_active`, `is_verified`, `cached_time`
impl Into<Value> for User {
    fn into(self) -> Value {
        object!({
            uid: self.id.uid,
            server: self.id.server.to_string(),
            username: self.username,
            email: self.email,
            is_active: self.is_active,
            is_verified: self.is_verified,
            cached_time: self.cached_at,
        })
    }
}

/// Convert a `User` into its JSON string representation for
/// low-level session storage.
impl Into<String> for User {
    fn into(self) -> String {
        let v: Value = self.into();
        v.into_json()
    }
} 

/// Globally unique user handle with numeric ID plus server origin.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct UserID {
    /// Numeric portion of the user ID
    pub uid: usize,
    /// Server or host where the user account lives
    pub server: Server,
}

impl UserID {
    /// Create a new `UserID` from parts.
    pub fn new(uid: usize, server: Server) -> Self {
        Self { uid, server }
    }

    /// Parse `"123@hostname"` into a `UserID(123, "hostname")`.
    pub fn from_str(s: &str) -> Option<Self> {
        let mut parts = s.splitn(2, '@');
        let raw_id = parts.next()?.parse().ok()?;
        let host = parts.next()?.to_string();
        Some(Self::new(raw_id, host.into()))
    } 

    pub fn is_guest(&self) -> bool {
        self.uid == 0
    } 
}

impl std::fmt::Display for UserID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.uid, self.server)
    }
} 

impl From<User> for UserID {
    fn from(auth_user: User) -> Self {
        UserID::new(auth_user.get_uid(), auth_user.get_server().clone())
    }
}  
