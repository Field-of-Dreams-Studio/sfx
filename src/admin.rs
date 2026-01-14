use hotaru::prelude::*; 
use hotaru::http::*; 

use crate::user::{UserID, fetch::get_user_id}; 
use crate::op; 
use crate::APP; 

pub mod api; 
pub mod panel; 
pub mod user; 

pub async fn check_is_admin(req: &mut HttpReqCtx) -> bool { 
    let user = object!(get_user_id(req).await.to_string());
    println!("check_is_admin: user: {}, admins: {}, is_admin: {}", user, op::get_admin(), op::get_admin().contains(&user)); 
    op::get_admin().contains(&user) 
}  

middleware! { 
    /// Middleware to redirect non-admin users to unauthorized page 
    /// **MUST ADD AFTER UserFetch MIDDLEWARE**  
    pub RedirectNonAdmin <HTTP> { 
        if check_is_admin(&mut req).await { 
            req.response = redirect_response("/user/unauthorized"); 
            return req 
        } else { 
            next(req).await 
        } 
    } 
} 

pub fn check_is_admin_id(id: UserID) -> bool {
    println!("check_is_admin_id: user: {}, admins: {}, is_admin: {}", id, op::get_admin(), op::get_admin().contains(&object!(id.to_string())));
    op::get_admin().contains(&object!(id.to_string()))
} 

endpoint! {
    APP.url("/admin/"),

    pub admin <HTTP> {
        if !check_is_admin(req).await { 
            return redirect_response("/user/unauthorized");
        }
        akari_render!(
            "admin/index.html", 
            pageprop = op::pageprop(req, "Admin", "Admin Dashboard"), 
            path = op::into_path_l(req, vec!["home", "admin"]), 
        ) 
    }
}
