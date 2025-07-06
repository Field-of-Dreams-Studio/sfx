use starberry::{HttpReqCtx, Value};

/// Represents a server or host where user accounts are stored. 
/// `Local` is a special case for local-only accounts, while `MainAuth` is for accounts managed by the main authentication server. 
#[derive(Debug, Clone, Hash, Eq, PartialEq)] 
pub enum Server {
    Local, 
    MainAuth(String) 
} 

impl Server { 
    /// Create a new `Server` from a string.
    /// If the string is `"local"`, it returns `Server::Local`.
    /// Otherwise, it returns `Server::MainAuth` with the string as the host.
    pub fn from_string(s: &str) -> Self { 
        if s == "local" {
            Server::Local
        } else {
            Server::MainAuth(s.to_string())
        }
    } 

    /// Get the server's host string. 
    /// 
    /// Returns the host string if this is `Server::MainAuth`, otherwise returns `"local"` for `Server::Local`.
    pub fn get_host(&self) -> &str {
        match self {
            Server::Local => "local",
            Server::MainAuth(host) => host,
        }
    } 

    /// Check if this server is the local one. 
    /// 
    /// Returns `true` if this is `Server::Local`, otherwise `false`.
    pub fn is_local(&self) -> bool {
        matches!(self, Server::Local)
    } 

    /// Get the actual address of the server. 
    pub fn get_address(&self) -> String { 
        if self.is_local() { 
            format!("http://{}", crate::op::APP.binding_address)
        } else {
            format!("https://{}", self.get_host())
        } 
    }
}

impl std::fmt::Display for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_host().fmt(f) 
    }
} 

impl Into<String> for Server {
    fn into(self) -> String {
        self.get_host().to_string() 
    }
} 

impl From<&str> for Server {
    fn from(s: &str) -> Self {
        Server::from_string(s) 
    }
} 

impl From<String> for Server {
    fn from(s: String) -> Self {
        Server::from_string(&s) 
    }
} 

impl From<Value> for Server { 
    fn from(value: Value) -> Self { 
        Self::from_string(&value.string()) 
    }
} 

impl Into<Value> for Server {
    fn into(self) -> Value {
        Value::from(self.get_host())
    }
} 
