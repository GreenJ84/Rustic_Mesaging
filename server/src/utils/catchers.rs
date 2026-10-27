use rocket::Request;

#[catch(401)]
pub fn unauthorized(req: &Request) -> String {
    String::from("Who are you? You can't be here unless we know you...")
}

#[catch(403)]
pub fn forbidden(req: &Request) -> String {
    String::from("We know who you are and you are not supposed to be here... Go away!")
}

#[catch(404)]
pub fn not_found(req: &Request) -> String {
    format!(
        "We can't seem to find what you are looking for at '{}'",
        req.uri()
    )
}

#[catch(405)]
pub fn method_not_allowed(req: &Request) -> String {
    format!(
        "We can't do '{}' for the requested resource '{}'",
        req.method(),
        req.uri()
    )
}

#[catch(408)]
pub fn request_timeout(req: &Request) -> String {
    format!(
        "We took too long to complete the request for '{}'",
        req.uri()
    )
}

#[catch(409)]
pub fn conflict(req: &Request) -> String {
    format!(
        "Yea, we can't complete the {} to '{}' because something is conflicting.",
        req.method(),
        req.uri()
    )
}

#[catch(424)]
pub fn failed_dependency(req: &Request) -> String {
    String::from("The operation failed because a dependency failed, not me.")
}

#[catch(500)]
pub fn internal_error() -> &'static str {
    "Whoops! Looks like I messed up."
}
