use actix_web::{Responder, post, HttpResponse, get, web};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
pub struct MessageRequest {
    to_user_id: i64,
    message: String,
}

#[post("/api/v1/messages")]
pub async fn send_message(req: actix_web::HttpRequest, message_req: web::Json<MessageRequest>) -> impl Responder {
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header.to_str().unwrap_or("").replace("Bearer ", ""),
        None => return HttpResponse::Unauthorized().json(json!({
            "success": false,
            "error": "Missing Authorization header"
        }))
    };

    let conn = match rusqlite::Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    let user_id: i64 = match conn.query_row(
        "SELECT id FROM accounts WHERE unique_id = ?1",
        [&auth_header],
        |row| row.get(0)
    ) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "success": false,
            "error": "Invalid authorization token"
        }))
    };

    match store_message(user_id, message_req.to_user_id, &message_req.message) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Message sent successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("Failed to send message: {}", e)
        }))
    }
}

#[get("/api/v1/messages")]
pub async fn get_user_messages(req: actix_web::HttpRequest) -> impl Responder {
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header.to_str().unwrap_or("").replace("Bearer ", ""),
        None => return HttpResponse::Unauthorized().json(json!({
            "success": false,
            "error": "Missing Authorization header"
        }))
    };

    let conn = match rusqlite::Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    let user_id: i64 = match conn.query_row(
        "SELECT id FROM accounts WHERE unique_id = ?1",
        [&auth_header],
        |row| row.get(0)
    ) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().json(json!({
            "success": false,
            "error": "Invalid authorization token"
        }))
    };

    match get_messages(user_id) {
        Ok(messages) => HttpResponse::Ok().json(json!({
            "success": true,
            "messages": messages
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("Failed to retrieve messages: {}", e)
        }))
    }
}