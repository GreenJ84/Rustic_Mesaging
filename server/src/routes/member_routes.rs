use rocket::form::{Form, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::State;

use crate::db::{DbPool, get_db_connection};
use crate::routes::CustomResponse;
use crate::service::{CrudOps, member_service::MemberService};
use crate::models::{
    authentication::JWT,
    member::{MemberSafe, NewMember},
    server::MultiServer,
    chat::MultiChatPreview,
    friend::MultiFriend,
    friend_request::MultiFriendRequest
};
use crate::models::authentication::Register;
use crate::models::friend::FullFriend;
use crate::models::member::{MemberShort, MultiMemberPreview};

#[head("/")]
pub fn head(
    _token: JWT,
) -> CustomResponse<()>{
    Ok(status::Custom(Status::Ok, ()))
}

/// Member creation ONLY through authenticating Registration route
#[get("/")]
pub fn get(
    token: JWT,
    pool: &State<DbPool>,
) -> CustomResponse<MemberSafe>{
    let mut conn = get_db_connection(pool)?;

    match MemberService::read(&mut conn, token.claims.member_id){
        Ok(member) => Ok(
            status::Custom(
                Status::Ok,
                member.into_safe()
            )),
        Err(e) => Err::<status::Custom<MemberSafe>, status::Custom<String>>(
            status::Custom(
                Status::NotFound,
                format!("Member lookup error: {}", e)
            ))
    }
}

#[put("/", data = "<form>")]
pub fn update(
    token: JWT,
    pool: &State<DbPool>,
    form: Form<Strict<Register>>,
) -> CustomResponse<MemberSafe> {
    let mut conn = get_db_connection(pool)?;
    let input = form.into_inner().into_inner();
    let member = MemberService::read(&mut conn, token.claims.member_id)
        .map_err(|e| { status::Custom(
            Status::NotFound,
            format!("Error finding member to update: {}", e)
        )})?;

    if input.password.ne(input.confirm_password){
        return Err::<status::Custom<MemberSafe>, status::Custom<String>>(
            status::Custom(
                Status::Unauthorized,
                String::from("Username or Password incorrect")
            ))
    }
    member.validate_password(input.confirm_password)
        .map_err(|e| { status::Custom(
            e.0, e.1
        )})?;

    match MemberService::update(&mut conn, token.claims.member_id, NewMember::new(
        input.username.to_string(),
        member.password().to_string(),
        input.email.to_string()
    )){
        Ok(member) => Ok(
            status::Custom(
                Status::Ok,
                member.into_safe()
            )),
        Err(e) =>
            return Err::<status::Custom<MemberSafe>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Member update error: {}", e)
            ))
    }
}

#[delete("/")]
pub fn delete(
    token: JWT,
    pool: &State<DbPool>,
) -> CustomResponse<String>{
    let mut conn = get_db_connection(pool)?;

    match MemberService::delete(&mut conn, token.claims.member_id){
        Ok(size) => {
            if size > 1 {
                eprint!("{} number of documents were deleted", size);
            }
            Ok(status::Custom(Status::Ok, format!("Member number {} has been deleted.", token.claims.member_id)))
        },
        Err(e) => {
            return Err::<status::Custom<String>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                   format!("Member deletion error: {}", e)
            ))
        }
    }
}

#[get("/<username>")]
pub fn get_by_username(
    _token: JWT,
    pool: &State<DbPool>,
    username: String,
) -> CustomResponse<MemberShort>{
    let mut conn = get_db_connection(pool)?;

    match MemberService::find_by_username(&mut conn, username){
        Ok(member) => Ok(
            status::Custom(
                Status::Ok,
                member.into_short()
            )),
        Err(e) =>
            return Err::<status::Custom<MemberShort>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Member search error: {}", e)
                ))
    }
}

#[get("/search?<search_term>")]
pub fn search_members(
    _token: JWT,
    pool: &State<DbPool>,
    search_term: String,
) -> CustomResponse<MultiMemberPreview>{
    let mut conn = get_db_connection(pool)?;

    match MemberService::search_usernames(&mut conn, search_term){
        Ok(members) => Ok(
            status::Custom(
                Status::Ok,
                MultiMemberPreview { members }
            )),
        Err(e) =>
            return Err::<status::Custom<MultiMemberPreview>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Member search error: {}", e)
                ))
    }
}

#[get("/chats")]
pub fn get_all_member_chats(
    token: JWT,
    pool: &State<DbPool>,
) -> CustomResponse<MultiChatPreview>{
    let mut conn = get_db_connection(pool)?;

    match MemberService::get_member_chats(&mut conn, token.claims.member_id){
        Ok(chats) => Ok(
            status::Custom(
                Status::Ok,
                MultiChatPreview { chat_previews: chats }
            )),
        Err(e) =>
            return Err::<status::Custom<MultiChatPreview>, status::Custom<String>>(
                status::Custom(
                    Status::NotFound,
                    format!("Member messages lookup error: {}", e)
                ))
    }
}

#[get("/servers")]
pub fn get_all_member_servers(
    token: JWT,
    pool: &State<DbPool>,
) -> CustomResponse<MultiServer>{
    let mut conn = get_db_connection(pool)?;

    match MemberService::get_member_servers(&mut conn, token.claims.member_id){
        Ok(servers) => Ok(
            status::Custom(
                Status::Ok,
                MultiServer { servers }
            )),
        Err(e) => Err::<status::Custom<MultiServer>, status::Custom<String>>(
            status::Custom(
                Status::NotFound,
                format!("Member servers lookup error: {}", e)
            ))
    }
}

#[get("/friends")]
pub fn get_all_friends(
    token: JWT,
    pool: &State<DbPool>,
) -> CustomResponse<MultiFriend>{
    let mut conn = get_db_connection(pool)?;

    match MemberService::get_member_friends(&mut conn, token.claims.member_id){
        Ok(friends) => {
            Ok(status::Custom(
                Status::Ok,
                MultiFriend { friends: friends.into_iter()
                    .map(|(created_at, member)|
                        FullFriend { member: member.into_short(), created_at}
                    ).collect::<Vec<FullFriend>>()
                },
            ))
        },
        Err(e) => Err::<status::Custom<MultiFriend>, status::Custom<String>>(
            status::Custom(
                Status::NotFound,
                format!("Member servers lookup error: {}", e)
            ))
    }
}


#[get("/requests")]
pub fn get_friend_requests(
    token: JWT,
    pool: &State<DbPool>,
) -> CustomResponse<MultiFriendRequest>{
    let mut conn = get_db_connection(pool)?;

    match MemberService::get_member_requests(&mut conn, token.claims.member_id){
        Ok(requests) => Ok(
            status::Custom(
                Status::Ok,
                requests
            )),
        Err(e) => Err::<status::Custom<MultiFriendRequest>, status::Custom<String>>(
            status::Custom(
                Status::NotFound,
                format!("Member friend requests lookup error: {}", e)
            ))
    }
}
