use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use rusqlite::{params, Connection, Result as SqliteResult};

#[derive(Deserialize)]
pub struct MessageRequest {
    receiver_id: i64,
    content: String,
    #[serde(default)]
    message_type: Option<String>,
    #[serde(default)] 
    file_url: Option<String>
}

#[derive(Serialize)]
struct Message {
    id: i64,
    content: String,
    sender_id: i64,
    receiver_id: i64,
    timestamp: String,
    read: bool,
    message_type: Option<String>,
    file_url: Option<String>
}

#[derive(Serialize)]
struct Conversation {
    id: i64,
    first_name: String,
    last_name: String,
    pfp: String,
    last_message: Option<LastMessage>,
}

#[derive(Serialize)]
struct LastMessage {
    content: String,
    timestamp: String,
    unread: bool,
}

fn store_message(user_id: i64, message_data: &str) -> SqliteResult<()> {
    let conn = Connection::open("fbla.db")?;
    conn.execute(
        "INSERT INTO messages (user_id, message_data, created_at) VALUES (?, ?, datetime('now'))",
        params![user_id, message_data]
    )?;
    Ok(())
}

fn fetch_messages(user_id: i64) -> SqliteResult<Vec<Message>> {
    let conn = Connection::open("fbla.db")?;
    let mut stmt = conn.prepare(
        "SELECT id, user_id, message_data, created_at 
         FROM messages 
         WHERE user_id = ?
         ORDER BY created_at DESC"
    )?;
    
    let messages = stmt.query_map([user_id], |row| {
        Ok(Message {
            id: row.get(0)?,
            content: row.get(1)?,
            sender_id: row.get(2)?,
            receiver_id: row.get(3)?,
            timestamp: row.get(4)?,
            read: row.get(5)?,
            message_type: row.get(6)?,
            file_url: row.get(7)?
        })
    })?;

    let mut result = Vec::new();
    for message in messages {
        result.push(message?);
    }
    Ok(result)
}

#[post("/api/v1/messages")]
pub async fn send_message(req: HttpRequest, message: web::Json<MessageRequest>) -> impl Responder {
    let token = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or("").replace("Bearer ", ""),
        None => return HttpResponse::Unauthorized().json(json!({"success": false})),
    };

    let conn = match Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"success": false})),
    };

    let sender_id: i64 = match conn.query_row(
        "SELECT id FROM accounts WHERE unique_id = ?",
        [&token],
        |row| row.get(0),
    ) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().json(json!({"success": false})),
    };

    match conn.execute(
        "INSERT INTO messages (sender_id, receiver_id, content, timestamp, read, message_type, file_url) 
         VALUES (?, ?, ?, datetime('now'), false, ?, ?)",
        params![
            sender_id,
            message.receiver_id,
            message.content,
            message.message_type,
            message.file_url
        ],
    ) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"success": false})),
    }
}

#[get("/api/v1/messages/{user_id}")]
pub async fn get_messages(req: HttpRequest, user_id: web::Path<i64>) -> impl Responder {
    let token = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or("").replace("Bearer ", ""),
        None => return HttpResponse::Unauthorized().json(json!({"success": false})),
    };

    let conn = match Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"success": false})),
    };

    let current_user_id: i64 = match conn.query_row(
        "SELECT id FROM accounts WHERE unique_id = ?",
        [&token],
        |row| row.get(0),
    ) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().json(json!({"success": false})),
    };

    let mut stmt = match conn.prepare(
        "SELECT id, content, sender_id, receiver_id, timestamp, read, message_type, file_url 
         FROM messages 
         WHERE (sender_id = ? AND receiver_id = ?) 
            OR (sender_id = ? AND receiver_id = ?)
         ORDER BY timestamp ASC"
    ) {
        Ok(stmt) => stmt,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"success": false})),
    };

    let messages_result = stmt.query_map(
        params![
            current_user_id,
            *user_id,
            *user_id,
            current_user_id
        ],
        |row| {
            Ok(Message {
                id: row.get(0)?,
                content: row.get(1)?,
                sender_id: row.get(2)?,
                receiver_id: row.get(3)?,
                timestamp: row.get(4)?,
                read: row.get(5)?,
                message_type: row.get(6)?,
                file_url: row.get(7)?
            })
        },
    );

    let messages = match messages_result {
        Ok(mapped) => {
            let collected: Result<Vec<Message>, _> = mapped.collect();
            match collected {
                Ok(msgs) => msgs,
                Err(_) => return HttpResponse::InternalServerError().json(json!({"success": false})),
            }
        },
        Err(_) => return HttpResponse::InternalServerError().json(json!({"success": false})),
    };

    // Mark messages as read
    conn.execute(
        "UPDATE messages SET read = true 
         WHERE sender_id = ? AND receiver_id = ? AND read = false",
        params![*user_id, current_user_id],
    ).ok();

    HttpResponse::Ok().json(json!({
        "success": true,
        "messages": messages
    }))
}

#[get("/api/v1/conversations")]
pub async fn get_conversations(req: HttpRequest) -> impl Responder {
    let token = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or("").replace("Bearer ", ""),
        None => return HttpResponse::Unauthorized().json(json!({"success": false})),
    };

    let conn = match Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"success": false})),
    };

    let current_user_id: i64 = match conn.query_row(
        "SELECT id FROM accounts WHERE unique_id = ?",
        [&token],
        |row| row.get(0),
    ) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().json(json!({"success": false})),
    };

    let mut stmt = match conn.prepare(
        "SELECT DISTINCT 
            a.id, 
            a.first_name, 
            a.last_name, 
            a.profile_picture,
            m.content,
            m.timestamp,
            m.read,
            m.sender_id = ? as is_sender
         FROM accounts a
         JOIN messages m ON (m.sender_id = a.id OR m.receiver_id = a.id)
         WHERE (m.sender_id = ? OR m.receiver_id = ?)
         AND a.id != ?
         ORDER BY m.timestamp DESC"
    ) {
        Ok(stmt) => stmt,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"success": false})),
    };

    let conversations_result = stmt.query_map(
        params![
            current_user_id,
            current_user_id,
            current_user_id,
            current_user_id
        ],
        |row| {
            Ok(Conversation {
                id: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                pfp: row.get(3)?,
                last_message: Some(LastMessage {
                    content: row.get(4)?,
                    timestamp: row.get(5)?,
                    unread: !row.get::<_, bool>(6)? && !row.get::<_, bool>(7)?,
                }),
            })
        },
    );

    let conversations = match conversations_result {
        Ok(mapped) => {
            let collected: Result<Vec<Conversation>, _> = mapped.collect();
            match collected {
                Ok(convs) => convs,
                Err(_) => return HttpResponse::InternalServerError().json(json!({"success": false})),
            }
        },
        Err(_) => return HttpResponse::InternalServerError().json(json!({"success": false})),
    };

    HttpResponse::Ok().json(json!({
        "success": true,
        "conversations": conversations
    }))
}
