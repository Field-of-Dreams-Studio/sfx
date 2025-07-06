use crate::user::{UserID, fetch::get_user_id}; 
use crate::op; 
use crate::APP; 

pub mod api; 
pub mod panel; 
pub mod user; 

use starberry::prelude::*; 


pub async fn check_is_admin(req: &mut HttpReqCtx) -> bool { 
    let user = object!(get_user_id(req).await.to_string());
    println!("check_is_admin: user: {}, admins: {}, is_admin: {}", user, op::get_admin(), op::get_admin().contains(&user)); 
    op::get_admin().contains(&user) 
} 


pub fn check_is_admin_id(id: UserID) -> bool {
    println!("check_is_admin_id: user: {}, admins: {}, is_admin: {}", id, op::get_admin(), op::get_admin().contains(&object!(id.to_string())));
    op::get_admin().contains(&object!(id.to_string()))
} 

#[url(APP.lit_url("/admin/"))] 
async fn admin() -> HttpResponse { 
    if !check_is_admin(req).await { 
        return redirect_response("/user/unauthorized")
    }; 
    akari_render!(
        "admin/index.html", 
        pageprop = op::pageprop(req, "Admin", "Admin Dashboard"), 
        path = op::into_path_l(req, vec!["home", "admin"]), 
    ) 
} 
