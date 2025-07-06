use sbmstd::session::CSessionRW;
use starberry::prelude::*; 

use super::fetch::*; 
use super::user::*; 
use super::{HALF_VALID_TIME, CACHE_VALID_TIME}; 

#[middleware]
async fn UserFetch() { 
    let auth_token = get_auth_token(&mut req);
    let host = get_host(&mut req); 
    if let None = auth_token {
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
            // handle "still valid" cache
            req.params.set::<User>(user); 
            let new_cache = fetch_user_info(host, auth_token.clone());
            new_cache.await.map(|new_user| {
                cache_user_info(&mut req, new_user);
            });
            redirect_refresh(&mut req); 
            return req // This is correct because redirect_refresh already write the request in context 
        }
        _ => {
            // cache expired
            match fetch_user_info(host.clone(), auth_token.clone()).await {
                Some(new_user) => {
                    req.params.set::<User>(new_user.clone());
                    cache_user_info(&mut req, new_user);
                    return next(req).await;
                }
                None => { 
                    req.params.set::<User>(User::guest(host));  
                    req.params
                        .get_mut::<CSessionRW>()
                        .unwrap()
                        .remove("user_info_cache");
                    return next(req).await; 
                }
            }
        }
    }
}

