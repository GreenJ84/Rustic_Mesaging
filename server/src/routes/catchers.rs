use rocket::Request;

#[catch(401)]
pub fn unauthorized_catcher(req: &Request) -> String {
    String::from("Unauthorized: Error validating member authentication")
}

#[catch(404)]
pub fn not_found(req: &Request) -> String {
    format!("I couldn't find '{}'. Try something else?", req.uri())
}

#[catch(409)]
pub fn conflict_catcher(req: &Request) -> String {
    String::from("Transaction conflict")
}

#[catch(424)]
pub fn failed_dependency(req: &Request) -> String {
    String::from("Failed transaction dependency")
}

#[catch(500)]
pub fn internal_error() -> &'static str {
    "Whoops! Looks like we messed up."
}