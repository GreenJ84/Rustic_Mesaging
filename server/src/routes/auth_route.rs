use rocket::form::{Form, Strict};
use rocket::http::Status;
use rocket::{State};
use rocket::response::status;
use crate::db::{DbPool, get_db_connection};
use crate::routes::CustomResponse;
use crate::service::member_service::MemberService;
use crate::models::authentication::*;
use crate::models::member::MemberSafe;
use crate::service::CrudOps;

#[post("/register", data = "<form>")]
pub fn register(
    pool: &State<DbPool>,
    form: Form<Strict<Register>>,
    admin: AdminKey,
) -> CustomResponse<AuthenticationResponse> {
    let mut conn = get_db_connection(pool)?;
    let input= form.into_inner().into_inner();

    let mut new_user = input.validate(&mut conn)
        .map_err(|e| {
            status::Custom(e.0, e.1)
        })?;

    if admin.passkey {
        new_user.set_admin();
    }

    let user = MemberService::create(&mut conn, new_user)
        .map_err(|e| { status::Custom(
                Status::InternalServerError,
                format!("Error creating new member: {}", e)
        )})?;

    // Send new MemberSafe with JWT
    match JWT::create_jwt(&user) {
        Ok(token) => Ok(status::Custom(Status::Created, AuthenticationResponse{ member: user.into_safe(), token })),
        Err(e) => Err::<status::Custom<AuthenticationResponse>, status::Custom<String>>(
            status::Custom(
                Status::InternalServerError,
                format!("Error creating Json Web Token: {}", e)
            )
        )
    }
}

#[post("/login", data="<form>")]
pub fn login(
    pool: &State<DbPool>,
    form: Form<Strict<Login>>
) -> CustomResponse<AuthenticationResponse> {
    let mut conn = get_db_connection(pool)?;
    let input = form.into_inner().into_inner();

    let member = MemberService::find_by_username(&mut conn, input.username.to_string())
        .map_err(|_| { status::Custom(
                Status::NotFound,
                String::from("Username or Password is incorrect")
        )})?;
    // Authenticate member
    match member.validate_password(input.password) {
        Ok(_) => {
            // Attach JWT to valid login
            match JWT::create_jwt(&member) {
                Ok(token) => Ok(status::Custom(Status::Ok, AuthenticationResponse { member: member.into_safe(), token })),
                Err(e) => Err::<status::Custom<AuthenticationResponse>, status::Custom<String>>(
                    status::Custom(
                        Status::InternalServerError,
                        format!("Internal server error: {}", e)
                    )
                )
            }
        },
        // Invalid login
        Err(e) => return Err(status::Custom(e.0, e.1))
    }
}

#[put("/password", data = "<form>")]
pub fn update_password(
    token: JWT,
    pool: &State<DbPool>,
    form: Form<Strict<Register>>,
) -> CustomResponse<MemberSafe> {
    let mut conn = get_db_connection(pool)?;
    let input= form.into_inner().into_inner();

    let member = MemberService::find_by_username(&mut conn, input.username.to_string())
        .map_err(|e| { status::Custom(
            Status::NotFound,
            format!("Error finding member to update: {}", e)
        )})?;

    if member.id() != token.claims.member_id {
        return Err::<status::Custom<MemberSafe>, status::Custom<String>>(status::Custom(
            Status::Forbidden,
            String::from("Never able to update another member")
        ));
    }
    member.validate_password(input.confirm_password).map_err(|e| { status::Custom(
        e.0,
        e.1
    )})?;

    match MemberService::update_password(&mut conn, token.claims.member_id, Register::hash_password(input.password)){
        Ok(member) => Ok(
            status::Custom(
                Status::Ok,
                member.into_safe()
            )),
        Err(e) =>
            Err::<status::Custom<MemberSafe>, status::Custom<String>>(
                status::Custom(
                    Status::InternalServerError,
                    format!("Member update error: {}", e)
                ))
    }
}