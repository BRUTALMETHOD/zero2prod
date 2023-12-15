use crate::utils::{e500, see_other};
use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

use crate::session_state::TypedSession;
pub async fn newsletter_form(
    session: TypedSession,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Add Newsletter</title>
</head>
<body>
    {msg_html}
    <form action="/newsletter" method="post">
        <label> 
            <input
                type="text"
                placeholder="blah here"
                name="newsletter_input"
            >
        </label>
        <button type="submit">Submit!</button>
</form>
    <p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>"#,
        )))
}
