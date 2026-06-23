use hotaru::prelude::*; 
use hotaru::http::*; 
use htmstd::session::CSessionRW;

use super::fetch::*; 
use super::user::*; 
use super::{HALF_VALID_TIME, CACHE_VALID_TIME}; 

middleware! {
    /// Middleware to fetch and cache user information based on auth token in session. 
    /// If no token is present, sets user as guest. 
    pub UserFetch <HTTP> { 
        let auth_token = get_auth_token(&req);
        let host = get_host(&req); 
        if auth_token.is_none() {
            req.params.set::<User>(User::guest(host));  
            return next(req).await;
        } 
        let auth_token = auth_token.unwrap(); 
        // println!("Cached: {:?}", req
        //     .params
        //     .get_mut::<CSessionRW>()
        //     .unwrap()
        //     .get("user_info_cache")); 
        let user = match req
            .params
            .get_mut::<CSessionRW>()
            .unwrap()
            .get("user_info_cache")
        { 
            Some(user) => user.clone().into(), 
            None => match fetch_user_info(host.clone(), auth_token.clone()).await {
                Some(user) => {
                    cache_user_info(&mut req, user.clone());
                    user
                }
                None => {
                    logout(&mut req).await;
                    req.params.set::<User>(User::guest(host.clone()));
                    cache_user_info(&mut req, User::guest(host));
                    return next(req).await
                }
            },
        }; 
        println!("User info: {:?}, Cached at: {}", user, user.cache_age()); 
        match user.cache_age() {
            0..HALF_VALID_TIME => {
                req.params.set::<User>(user);
                return next(req).await;
            }
            HALF_VALID_TIME..=CACHE_VALID_TIME => {
                // Cache is half-valid: serve it, refresh in background.
                req.params.set::<User>(user);
                match fetch_user_info(host.clone(), auth_token.clone()).await {
                    Some(new_user) => {
                        cache_user_info(&mut req, new_user);
                        return next(req).await;
                    }
                    None => {
                        // The stored token no longer validates (server restart,
                        // manual revocation, TTL eviction, etc.). Redirecting to
                        // /user/refresh would loop because /auth/refresh hits the
                        // same failing token. Drop the session and continue as
                        // guest so the handler can decide what to do.
                        logout(&mut req).await;
                        req.params.set::<User>(User::guest(host));
                        return next(req).await;
                    }
                }
            }
            _ => {
                // Cache expired entirely.
                match fetch_user_info(host.clone(), auth_token.clone()).await {
                    Some(new_user) => {
                        req.params.set::<User>(new_user.clone());
                        cache_user_info(&mut req, new_user);
                        return next(req).await;
                    }
                    None => {
                        // Same as the half-valid case: token is dead; clear it
                        // so the next request doesn't reload the loop.
                        logout(&mut req).await;
                        req.params.set::<User>(User::guest(host));
                        return next(req).await;
                    }
                }
            }
        }
    }
}
