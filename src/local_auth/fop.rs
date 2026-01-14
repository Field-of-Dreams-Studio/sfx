//! An in-memory authentication manager with periodic disk persistence.
//!
//! This module provides `AuthManager`, which loads all users from a JSON file on
//! startup and keeps them in memory. Any change (register, delete, password update)
//! is applied in-memory and flushed back to disk every 300 seconds (configurable).
//! The tokenlist lives purely in memory (no file). All methods are async and
//! use Tokio for the background flush task.
//!
//! # Example
//!
//! ```c 
//! use std::time::Duration;
//! use sfx::local_auth::AuthManager;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a manager that flushes every 5 minutes
//!     let manager = AuthManager::new("programfiles/local_auth/users.json", Duration::from_secs(300));
//!
//!     // Register a new user
//!     assert!(manager.register_user("alice", "secret", "alice@example.com").await?);
//!
//!     // Authenticate with correct password
//!     assert!(manager.authenticate_user("alice", "secret").await?);
//!
//!     // Authenticate with wrong password
//!     assert!(!manager.authenticate_user("alice", "wrong").await?);
//!
//!     // List usernames
//!     let names = manager.list_usernames().await;
//!     assert_eq!(names, vec!["alice".to_string()]);
//!
//!     Ok(())
//! }
//! ```
//!
use hotaru::prelude::*;
use hotaru_lib::ende::aes; 
use hotaru_lib::random::random_alphanumeric_string; 
use std::num::NonZeroU32; 
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use tokio::time; 

const DEFAULT_ITER: NonZeroU32 = NonZeroU32::new(100_000).unwrap(); 

/// A user record stored in memory.
#[derive(Clone, Debug)]
pub struct UserStorage { 
    pub username: String, 
    pub email: String, 
    pub password_hash: String,
    pub password_salt: String,
    pub profile: Value, 
}

impl UserStorage {
    fn from_json(value: Value) -> Self {
        UserStorage {
            username: value.get("username").string(),
            email: value.get("email").string(), 
            password_hash: value.get("password_hash").string(),
            password_salt: value.get("password_salt").string(),
            profile: value.get("profile").clone() 
        }
    }

    fn into_json(&self) -> Value {
        object!({
            username: &self.username, 
            email: &self.email, 
            password_hash: &self.password_hash,
            password_salt: &self.password_salt,
            profile: self.profile.clone() 
        })
    } 

    fn into_json_without_password(&self) -> Value {
        object!({
            username: &self.username, 
            email: &self.email, 
            profile: self.profile.clone() 
        })
    } 
} 

pub struct TokenList(RwLock<HashMap<String, (u32, u64)>>); // token -> (uid, expires) 

impl TokenList { 
    pub fn new() -> Self {
        TokenList(RwLock::new(HashMap::new()))
    } 

    /// Add a token to the list with user id and expiration time 
    pub async fn add(&self, token: String, uid: u32, expires: u64) {
        self.0.write().await.insert(token, (uid, expires));
    }

    /// Remove a token from the list 
    pub async fn remove(&self, token: &str) {
        self.0.write().await.remove(token);
    }

    /// Get the user's id by using the token 
    pub async fn authenticate_user(&self, token: &str) -> Option<u32> {
        let guard = self.0.read().await;
        if let Some(&(uid, expires)) = guard.get(token) {
            if expires > std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() {
                return Some(uid);
            }
        }
        None
    } 

    /// Search through all tokens and cleans up those are expired 
    pub async fn cleanup_expired(&self) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let mut guard = self.0.write().await;
        guard.retain(|_, &mut (_, expires)| expires > now);
    } 
} 

#[cfg(test)]
mod tests {
    use super::TokenList;
    use std::{
        collections::HashMap, 
        time::{SystemTime, UNIX_EPOCH},
    };
    use tokio::sync::RwLock;

    // Helper to get current unix timestamp in seconds
    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs()
    }

    #[tokio::test]
    async fn test_add_and_authenticate() {
        let list = TokenList(RwLock::new(HashMap::new()));
        let token = "token123".to_string();
        let uid = 42;
        let expires = now_secs() + 100;
        list.add(token.clone(), uid, expires).await;

        // Should authenticate while unexpired
        assert_eq!(list.authenticate_user(&token).await, Some(uid));
    }

    #[tokio::test]
    async fn test_expired_token() {
        let list = TokenList(RwLock::new(HashMap::new()));
        let token = "token_exp".to_string();
        let uid = 7;
        let expires = now_secs() - 1; // already expired
        list.add(token.clone(), uid, expires).await;

        // Should not authenticate expired token
        assert_eq!(list.authenticate_user(&token).await, None);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let list = TokenList(RwLock::new(HashMap::new()));
        let good = "good".to_string();
        let bad = "bad".to_string();
        let uid1 = 1;
        let uid2 = 2;
        let expires1 = now_secs() + 50;
        let expires2 = now_secs() - 50;
        list.add(good.clone(), uid1, expires1).await;
        list.add(bad.clone(), uid2, expires2).await;

        // Before cleanup: good authenticates, bad does not
        assert_eq!(list.authenticate_user(&good).await, Some(uid1));
        assert_eq!(list.authenticate_user(&bad).await, None);

        // Cleanup expired entries
        list.cleanup_expired().await;

        // Underlying map should only contain the good token
        let guard = list.0.read().await;
        assert!(guard.contains_key(&good));
        assert!(!guard.contains_key(&bad));
    }

    #[tokio::test]
    async fn test_remove_token() {
        let list = TokenList(RwLock::new(HashMap::new()));
        let token = "toremove".to_string();
        let uid = 3;
        let expires = now_secs() + 100;

        list.add(token.clone(), uid, expires).await;
        assert_eq!(list.authenticate_user(&token).await, Some(uid));

        // Remove and then fail to authenticate
        list.remove(&token).await;
        assert_eq!(list.authenticate_user(&token).await, None);
    }
} 

/// The authentication manager.
///
/// Loads users from disk once at startup, keeps them in memory,
/// and periodically flushes changes back to the JSON file.
/// Blacklist is kept only in memory.
pub struct AuthManager {
    users: Arc<RwLock<HashMap<u32, UserStorage>>>, 
    username_map: Arc<RwLock<HashMap<String, u32>>>, 
    email_map: Arc<RwLock<HashMap<String, u32>>>, 
    token_list: Arc<TokenList>, 
    path: String,
    max_uid: Arc<RwLock<u32>> 
} 

impl AuthManager { 
    /// Create a new `AuthManager` that reads `users_file` on startup and
    /// spawns a background task to flush every `interval`.
    pub fn new(users_file: impl Into<String>, interval: Duration) -> Self {
        let path = users_file.into(); 
        let mut user_map: HashMap<u32, UserStorage> = HashMap::new(); 
        let mut username_map: HashMap<String, u32> = HashMap::new(); 
        let mut email_map: HashMap<String, u32> = HashMap::new(); 
        let mut max_uid = 0_u32; 

        // Load users once
        if let Ok(Value::Dict(initial)) = Value::from_jsonf(&path) { 
            initial.into_iter().for_each(|(uid, value)| { 
                if let Ok(uid) = uid.parse::<u32>(){ 
                    let user_storage: UserStorage = UserStorage::from_json(value); 
                    username_map.insert(user_storage.username.clone(), uid); 
                    email_map.insert(user_storage.email.clone(), uid); 
                    user_map.insert(uid, user_storage); 
                    if max_uid < uid { 
                        max_uid = uid 
                    }
                }; 
            });
        }

        let users = Arc::new(RwLock::new(user_map));
        let username_map = Arc::new(RwLock::new(username_map)); 
        let email_map = Arc::new(RwLock::new(email_map));
        let token_list = Arc::new(TokenList::new());
        let users_clone = Arc::clone(&users); 
        let token_clone = Arc::clone(&token_list); 
        let path_clone = path.clone(); 

        // Spawn periodic flush
        let _flush_task = tokio::spawn(async move {
            let mut ticker = time::interval(interval);
            loop {
                ticker.tick().await;
                let guard = users_clone.read().await;
                let list = Value::Dict(guard.iter().map(|(uid, value)| (uid.to_string(), value.into_json())).collect());
                if let Err(err) = list.into_jsonf(&path_clone) {
                    eprintln!("Failed to flush users to {}: {}", &path_clone, err);
                } 
                token_clone.cleanup_expired().await; // Clean up expired tokens periodically 
            }
        });

        AuthManager { users, username_map, email_map, token_list, path, max_uid: Arc::new(RwLock::new(max_uid)) }
    }

    /// Use the uid to auth the user 
    pub async fn check_password(&self, uid: u32, password: &str) -> bool {
        let guard = self.users.read().await;
        if let Some(user) = guard.get(&uid) {
            // println!("{:?}", aes::decrypt(&user.password_hash, &user.password_salt)); 
            if aes::decrypt(&user.password_hash, &user.password_salt) == Ok(password.to_string()) { 
                return true 
            }
            false 
        } else {
            false 
        }
    } 

    /// Get the uid by using auth token 
    pub async fn authenticate_user(&self, token: &str) -> Result<Value, FopError> {
        if let Some(uid) = self.token_list.authenticate_user(token).await {
            let guard = self.users.read().await;
            if let Some(user) = guard.get(&uid) {
                Ok(user.into_json())
            } else {
                Err(FopError::UserNotFound)
            }
        } else {
            Err(FopError::TokenInvalid)
        }
    } 

    /// Login the user while generating a token for the user
    pub async fn login_user(&self, uid: u32, password: &str) -> Result<String, FopError> {
        println!("[AuthManager::login_user] Checking password for uid: {}", uid);
        if self.check_password(uid, password).await {
            let token = random_alphanumeric_string(32);
            let expires = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 3600; // 1 hour
            println!("[AuthManager::login_user] Generated token: {}, expires: {}", token, expires);
            self.token_list.add(token.clone(), uid, expires).await;
            println!("[AuthManager::login_user] Token added to token_list");
            Ok(token)
        } else {
            println!("[AuthManager::login_user] Password mismatch");
            Err(FopError::PasswordMismatch)
        }
    } 

    /// Logout the user by removing the token 
    pub async fn logout_user(&self, token: &str) -> Result<(), FopError> {
        if self.token_list.authenticate_user(token).await.is_some() {
            self.token_list.remove(token).await;
            Ok(())
        } else {
            Err(FopError::TokenInvalid)
        }
    } 

    /// Find the uid by using email 
    pub async fn get_uid_by_email(&self, email: &str) -> Option<u32> { 
        let guard = self.email_map.read().await; 
        guard.get(email).cloned() 
    } 

    /// Refresh a new token by using a old token
    /// The old token should be valid
    pub async fn refresh_token(&self, old_token: &str) -> Result<String, FopError> {
        if let Some(uid) = self.token_list.authenticate_user(old_token).await {
            let new_token = random_alphanumeric_string(32);
            let expires = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 3600; // 1 hour
            self.token_list.add(new_token.clone(), uid, expires).await;
            Ok(new_token)
        } else {
            Err(FopError::TokenInvalid)
        }
    }

    /// Find the uid by username 
    pub async fn get_uid_by_username(&self, username: &str) -> Option<u32> { 
        let guard = self.username_map.read().await; 
        guard.get(username).cloned() 
    } 

    /// Get the uid info by using one of the identification method  
    pub async fn uid_from_username_or_email_or_uid(&self, string: String) -> Result<u32, FopError> {
        if let Ok(uid) = string.parse::<u32>() {
            return Ok(uid);
        }
        if let Some(uid) = self.get_uid_by_email(&string).await {
            return Ok(uid);
        }
        if let Some(uid) = self.get_uid_by_username(&string).await {
            return Ok(uid);
        }
        Err(FopError::UserNotFound)
    } 

    /// Make sure the username have the following property 
    /// - It starts with a alphabetical character (not numerical) 
    /// - Any character in the username should be either alphabetical, numerical or within [",", ".", "_", "+", "-", "(", ")", "[", "]", "{", "}", "|"] 
    /// - It should not conflict with other usernames 
    pub async fn validate_username(&self, username: &str) -> bool { 
        println!("Validating username: {}/", username);
        // Rule #1: non-empty and first char is ASCII letter
        let mut chars = username.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() => {}
            _ => return false,
        }

        // Rule #2: every char must be allowed
        for c in username.chars() {
            if c.is_ascii_alphanumeric() {
                continue;
            }
            // allowed punctuation set
            match c {
                ',' | '.' | '_' | '+' | '-' |
                '(' | ')' | '[' | ']' | '{' |
                '}' | '|' => continue,
                _ => return false,
            }
        }

        // Rule #3: must not already exist
        let usernames = self.username_map.read().await;
        println!("Checking against existing usernames: {:?}", usernames);
        !usernames.contains_key(username)
    } 

    /// Validate an email address according to the following rules:
    /// 1. It must start with an ASCII alphabetic character (A–Z or a–z).
    /// 2. It must contain exactly one '@'.
    /// 3. The local-part (before '@') and the domain-part (after '@') may only contain:
    ///    - ASCII alphanumeric (A–Z, a–z, 0–9)
    ///    - one of the punctuation: , . _ + - ( ) [ ] { } |
    /// 4. It must not conflict with any existing email in the in-memory map.
    pub async fn validate_email(&self, email: &str) -> bool {
        let mut chars = email.chars();
        // Rule #1: non-empty and first char is ASCII letter
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() => {}
            _ => return false,
        }
        // Rule #2: exactly one '@'
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            println!("Length of parts: {}, {:?}", parts.len(), parts); 
            return false;
        }
        // Validate each side
        for part in parts.iter() {
            if part.is_empty() {
                return false;
            }
            for c in part.chars() {
                if c.is_ascii_alphanumeric() {
                    continue;
                }
                match c {
                    ',' | '.' | '_' | '+' | '-' |
                    '(' | ')' | '[' | ']' | '{' |
                    '}' | '|' => continue,
                    _ => return false,
                }
            }
        }
        // Rule #4: must not already exist
        let emails = self.email_map.read().await;
        !emails.contains_key(email)
    } 

    /// Generate a new uid where increasing max uid 
    pub async fn new_uid(&self) -> u32 { 
        let mut max_uid = self.max_uid.write().await;
        *max_uid += 1;
        *max_uid 
    } 

    /// Change the username 
    pub async fn change_username(&self, token: &str, new_username: &str) -> Result<(), FopError> { 
        let uid = match self.token_list.authenticate_user(token).await {
            Some(uid) => uid,
            None => return Err(FopError::TokenInvalid),
        }; 
        if !self.validate_username(new_username).await {
            return Err(FopError::UserNameNotValid);
        }
        let mut username_map = self.username_map.write().await;
        if let Some(old_username) = username_map.iter().find(|(_, v)| v == &&uid).map(|(k, _)| k.clone()) {
            username_map.remove(&old_username);
            username_map.insert(new_username.to_string(), uid); 
        } else {
            return Err(FopError::UserNotFound)
        } 
        let mut users = self.users.write().await; 
        if let Some(user) = users.get_mut(&uid) {
            user.username = new_username.to_string();
            Ok(())
        } else {
            Err(FopError::UserNotFound)
        } 
    } 

    /// Change the email 
    pub async fn change_email(&self, token: &str, new_email: &str) -> Result<(), FopError> {
        let uid = match self.token_list.authenticate_user(token).await {
            Some(uid) => uid,
            None => return Err(FopError::TokenInvalid),
        }; 
        if !self.validate_email(new_email).await {
            return Err(FopError::EmailNotValid);
        }
        let mut email_map = self.email_map.write().await;
        if let Some(old_email) = email_map.iter().find(|(_, v)| v == &&uid).map(|(k, _)| k.clone()) {
            email_map.remove(&old_email);
            email_map.insert(new_email.to_string(), uid);
        } else {
            return Err(FopError::UserNotFound);
        }
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(&uid) {
            user.email = new_email.to_string();
            Ok(())
        } else {
            Err(FopError::UserNotFound)
        }
    } 

    /// Change the password for a user 
    pub async fn change_password(&self, token: &str, old_password: &str, new_password: &str) -> Result<(), FopError> {
        let uid = match self.token_list.authenticate_user(token).await {
            Some(uid) => uid,
            None => return Err(FopError::TokenInvalid),
        }; 
        if self.check_password(uid, old_password).await {
            return Err(FopError::PasswordMismatch);
        } 
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(&uid) {
            user.password_hash = aes::encrypt(new_password, &user.password_salt).unwrap(); // Use the existing salt 
            Ok(())
        } else {
            Err(FopError::UserNotFound)
        }
    } 

    /// Register a new user 
    pub async fn register_user(&self, username: &str, email: &str, password: &str) -> Result<(), FopError> { 
        if !self.validate_username(username).await { 
            return Err(FopError::UserNameNotValid)
        }; 
        if !self.validate_email(email).await { 
            return Err(FopError::EmailNotValid)
        }; 
        let new_uid = self.new_uid().await; 
        self.username_map.write().await.insert(username.to_string(), new_uid); 
        self.email_map.write().await.insert(email.to_string(), new_uid); 
        let salt = random_alphanumeric_string(16); // Generate a random salt 
        let user = UserStorage { 
            username: username.to_string(), 
            email: email.to_string(), 
            password_hash: aes::encrypt(password, &salt).unwrap(), // Use a random salt
            password_salt: salt, 
            profile: object!({}) 
        }; 
        self.users.write().await.insert(new_uid, user); 
        Ok(()) 
    } 

    /// Change a user's info 
    pub async fn edit_user(&mut self, token: String, user: UserStorage) -> Result<(), FopError> { 
        match self.token_list.authenticate_user(&token).await { 
            Some(uid) => { 
                if !self.validate_username(&user.username).await { 
                    return Err(FopError::UserNameNotValid)
                }; 
                if !self.validate_email(&user.email).await { 
                    return Err(FopError::EmailNotValid)
                }; 
                let mut users = self.users.write().await; 
                if let Some(existing_user) = users.get_mut(&uid) { 
                    existing_user.username = user.username; 
                    existing_user.email = user.email; 
                    existing_user.password_hash = user.password_hash; 
                    existing_user.password_salt = user.password_salt; 
                    existing_user.profile = user.profile; 
                    Ok(())
                } else {
                    Err(FopError::UserTooBig)
                }
            },
            None => return Err(FopError::TokenInvalid), 
        } 
    } 

    /// Get user info 
    pub async fn get_user_profile(&mut self, token: String) -> Result<Value, FopError> { 
        match self.token_list.authenticate_user(&token).await { 
            Some(auth_uid) => { 
                let users = self.users.read().await; 
                if let Some(user) = users.get(&auth_uid) { 
                    Ok(user.profile.clone())
                } else {
                    Err(FopError::UserTooBig)
                }
            },
            _ => Err(FopError::TokenInvalid), 
        } 
    } 

    pub async fn get_userstorage_instance(self, token: String) -> Result<UserStorage, FopError> { 
        match self.token_list.authenticate_user(&token).await { 
            Some(auth_uid) => { 
                let users = self.users.read().await; 
                if let Some(user) = users.get(&auth_uid) { 
                    Ok(user.clone())
                } else {
                    Err(FopError::UserTooBig)
                }
            },
            _ => Err(FopError::TokenInvalid), 
        } 
    } 

    pub async fn get_user_info(&self, token: String) -> Result<Value, FopError> {
        println!("[AuthManager::get_user_info] Looking up token: {}", token);
        match self.token_list.authenticate_user(&token).await {
            Some(auth_uid) => {
                println!("[AuthManager::get_user_info] Token valid, uid: {}", auth_uid);
                let users = self.users.read().await;
                if let Some(user) = users.get(&auth_uid) {
                    println!("[AuthManager::get_user_info] Found user: {}", user.username);
                    Ok(object!({
                        username: &user.username,
                        email: &user.email,
                        uid: auth_uid
                    }))
                } else {
                    println!("[AuthManager::get_user_info] User not found for uid: {}", auth_uid);
                    Err(FopError::UserTooBig)
                }
            },
            _ => {
                println!("[AuthManager::get_user_info] Token not found in token_list");
                Err(FopError::TokenInvalid)
            },
        }
    } 

    pub async fn list_users(&self) -> Vec<Value> {
        let users = self.users.read().await;
        users.values().map(|user| user.into_json_without_password()).collect()
    } 
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FopError { 
    TooManyRequest, 
    UserNameNotValid, 
    EmailNotValid, 
    PasswordMismatch, 
    UserTooBig, 
    UserNotFound, 
    TokenInvalid, 
    Other(Box<str>) 
} 

impl ToString for FopError {
    fn to_string(&self) -> String {
        match self {
            FopError::TooManyRequest => "Too many requests".to_string(),
            FopError::UserNameNotValid => "Username is not valid".to_string(),
            FopError::EmailNotValid => "Email is not valid".to_string(),
            FopError::PasswordMismatch => "Password mismatch".to_string(),
            FopError::UserTooBig => "User data too big".to_string(),
            FopError::UserNotFound => "User not found".to_string(), 
            FopError::TokenInvalid => "Token is invalid".to_string(),
            FopError::Other(msg) => msg.to_string(),
        }
    }
} 

#[cfg(test)] 
mod test {
    use std::collections::HashMap;
    use tokio::sync::RwLock; 

    use hotaru::prelude::*; 
    use hotaru_lib::ende::aes; 

    use crate::local_auth::fop::AuthManager; 
    use crate::local_auth::fop::TokenList;
    use crate::local_auth::fop::UserStorage; 

    #[test] 
    pub fn test_user_from_json() { 
        let user = UserStorage::from_json(object!({
            username: "Admin", 
            email: "redstone@fds.moe", 
            password_hash: "js", 
            password_salt: "suki" 
        })); 
        assert_eq!(user.username, "Admin"); 
        assert_eq!(user.email, "redstone@fds.moe"); 
        assert_eq!(user.password_hash, "js"); 
        assert_eq!(user.password_salt, "suki"); 
    } 

    #[test] 
    pub fn test_user_into_json() { 
        let user = UserStorage { 
            username: "Admin".to_string(), 
            email: "redstone@fds.moe".to_string(), 
            password_hash: "123456".to_string(), 
            password_salt: "Aa333333".to_string(), 
            profile: object!({}) 
        }; 
        let value = user.into_json(); 
        println!("{}, {}", value.to_string(), value.into_json()) 
    }
 
    #[tokio::test] 
    pub async fn test_auth_user() { 

        let mut users = HashMap::new(); 
        users.insert(1_u32, UserStorage::from_json(object!({
            username: "Admin", 
            email: "redstone@fds.moe", 
            password_hash: aes::encrypt("js", "suki").unwrap(), 
            password_salt: "suki" 
        }))); 
        users.insert(2_u32, UserStorage::from_json(object!({
            username: "App", 
            email: "Sabi", 
            password_hash: aes::encrypt("ustc", "aes").unwrap(), 
            password_salt: "aes" 
        }))); 

        let mut username_map = HashMap::new(); 
        username_map.insert("Admin".to_string(), 1_u32); 
        username_map.insert("App".to_string(),2_u32); 

        let mut email_map = HashMap::new(); 
        email_map.insert("redstone@fds.moe".to_string(), 1_u32); 
        email_map.insert("Sabi".to_string(), 2_u32); 

        // Note that this auth manager have no ability to flush because it didn't use the new function 
        let auth = AuthManager { 
            users: Arc::new(RwLock::new(users)), 
            username_map: Arc::new(RwLock::new(username_map)), 
            email_map: Arc::new(RwLock::new(email_map)), 
            token_list: Arc::new(TokenList::new()),
            path: "test.json".to_string(),
            max_uid: Arc::new(RwLock::new(2_u32))
        };

        assert!(auth.check_password(1, "js").await); 
    } 
} 
