use hotaru::prelude::*; 
use hotaru::http::*; 

pub fn get_auth_token(req: &mut HttpReqCtx) -> Option<String> {
    let bearer_token = req.meta().get_header("Authorization")?;
    let token_str = bearer_token.strip_prefix("Bearer ")?;
    if token_str.is_empty() {
        None
    } else {
        Some(token_str.to_string())
    }
} 
