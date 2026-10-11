use rocket::form::{Form, Strict};
use rocket::http::Status;
use rocket::response::status;
use rocket::{Request, State};

use crate::auth::password::PasswordManager;
use crate::db::{get_db_connection, DbPool};

use super::error::AuthError;
use super::models::*;

use crate::api_error::ApiError;
use crate::member::models::MemberSafe;
use crate::member::service::MemberService;
use crate::routes::{ApiResponse, CustomResponse};
use crate::service::CrudOps;

#[post("/register", data = "<form>")]
pub fn register(
    pool: &State<DbPool>,
    form: Form<Strict<RegisterInput>>,
    admin: AdminKey,
) -> ApiResponse<AuthenticationResponse> {
    let mut conn = get_db_connection(pool).map_err(|e| ApiError::DatabaseError(e.1))?;

    let input = form.into_inner().into_inner();
    let mut new_user = input.validate(&mut conn)?;
    if admin.passkey {
        new_user.set_admin();
    }

    let user = MemberService::create(&mut conn, new_user).map_err(|e| ApiError::InternalError)?;

    // Send new MemberSafe with JWT
    match JWT::create_jwt(&user) {
        Ok(token) => Ok(status::Custom(
            Status::Created,
            AuthenticationResponse {
                member: user.into_safe(),
                token,
            },
        )),
        Err(_) => Err(ApiError::InternalError),
    }
}

#[post("/login", data = "<form>")]
pub fn login(
    pool: &State<DbPool>,
    form: Form<Strict<LoginInput>>,
) -> ApiResponse<AuthenticationResponse> {
    let mut conn = get_db_connection(pool).map_err(|e| ApiError::DatabaseError(e.1))?;

    let input = form.into_inner().into_inner();
    let member = MemberService::find_by_username(&mut conn, input.username.to_string())
        .map_err(|_| AuthError::AccountNotFound)?;

    // Authenticate member
    PasswordManager::verify(&input.password, member.password())?;

    // Attach JWT to valid login
    let token = JWT::create_jwt(&member)?;
    Ok(status::Custom(
        Status::Ok,
        AuthenticationResponse {
            member: member.into_safe(),
            token,
        },
    ))
}

#[put("/password", data = "<form>")]
pub fn update_password(
    token: JWT,
    pool: &State<DbPool>,
    form: Form<Strict<RegisterInput>>,
) -> ApiResponse<MemberSafe> {
    let mut conn = get_db_connection(pool).map_err(|e| ApiError::DatabaseError(e.1))?;

    let input = form.into_inner().into_inner();
    let member = MemberService::find_by_username(&mut conn, input.username.to_string())
        .map_err(|e| AuthError::AccountNotFound)?;

    if member.id() != token.claims.member_id {
        return Err(ApiError::from(AuthError::InvalidCredentials));
    } else if input.password.ne(&input.confirm_password) {
        return Err(ApiError::from(AuthError::PasswordConfirmationMismatch));
    }
    PasswordManager::verify(&input.password, member.password())?;

    match MemberService::update_password(
        &mut conn,
        token.claims.member_id,
        PasswordManager::hash(&input.password)?,
    ) {
        Ok(member) => Ok(status::Custom(Status::Ok, member.into_safe())),
        Err(e) => Err(ApiError::DatabaseError(e.to_string())),
    }
}
