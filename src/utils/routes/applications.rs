use actix_web::{post, get, put, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use rusqlite::params;

#[derive(Serialize, Deserialize)]
pub struct Application {
    pub id: Option<i64>,
    pub post_id: i64,
    pub applicant_id: String,
    pub employer_id: String,
    pub status: String,
    pub answers: String,
    pub created_at: String,
    pub updated_at: String
}

#[derive(Deserialize)]
pub struct CreateApplicationRequest {
    pub post_id: i64,
    pub answers: serde_json::Value
}

#[derive(Deserialize)]
pub struct UpdateApplicationStatusRequest {
    pub status: String
}

// Create a new application
#[post("/api/v1/apply")]
pub async fn create_application(req: HttpRequest, req_body: web::Json<CreateApplicationRequest>) -> impl Responder {
    // Get applicant ID from token
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header.to_str().unwrap_or("").replace("Bearer ", ""),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "Missing authorization header"
        }))
    };

    let conn = match rusqlite::Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    // Get employer_id from post
    let employer_id: String = match conn.query_row(
        "SELECT employer_id FROM posts WHERE id = ?1",
        [&req_body.post_id],
        |row| row.get(0)
    ) {
        Ok(id) => id,
        Err(_) => return HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": "Post not found"
        }))
    };

    let current_time = chrono::Utc::now().to_rfc3339();

    match conn.execute(
        "INSERT INTO applications (post_id, applicant_id, employer_id, status, answers, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            req_body.post_id,
            auth_header,
            employer_id,
            "pending",
            req_body.answers.to_string(),
            current_time,
            current_time
        ],
    ) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Application submitted successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to submit application: {}", e)
        }))
    }
}

// Get applications submitted by a user
#[get("/api/v1/applications/submitted")]
pub async fn get_submitted_applications(req: HttpRequest) -> impl Responder {
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header.to_str().unwrap_or("").replace("Bearer ", ""),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "Missing authorization header"
        }))
    };

    let conn = match rusqlite::Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    let mut stmt = match conn.prepare(
        "SELECT a.*, p.title as post_title, p.company_name 
         FROM applications a 
         JOIN posts p ON a.post_id = p.id 
         WHERE a.applicant_id = ?1"
    ) {
        Ok(stmt) => stmt,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    let applications: Result<Vec<serde_json::Value>, rusqlite::Error> = stmt
        .query_map([&auth_header], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>("id")?,
                "post_id": row.get::<_, i64>("post_id")?,
                "post_title": row.get::<_, String>("post_title")?,
                "company_name": row.get::<_, String>("company_name")?,
                "status": row.get::<_, String>("status")?,
                "answers": serde_json::from_str::<serde_json::Value>(&row.get::<_, String>("answers")?).unwrap_or(serde_json::json!({})),
                "created_at": row.get::<_, String>("created_at")?,
                "updated_at": row.get::<_, String>("updated_at")?
            }))
        })
        .and_then(Iterator::collect);

    match applications {
        Ok(apps) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "applications": apps
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to fetch applications: {}", e)
        }))
    }
}

// Get applications received by an employer
#[get("/api/v1/applications/received")]
pub async fn get_received_applications(req: HttpRequest) -> impl Responder {
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header.to_str().unwrap_or("").replace("Bearer ", ""),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "Missing authorization header"
        }))
    };

    let conn = match rusqlite::Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    // Verify user is an employer
    let account_type: String = match conn.query_row(
        "SELECT account_type FROM accounts WHERE unique_id = ?1",
        [&auth_header],
        |row| row.get(0)
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "Invalid authorization token"
        }))
    };

    if account_type != "employer" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "error": "Only employers can view received applications"
        }));
    }

    let mut stmt = match conn.prepare(
        "SELECT a.*, p.title as post_title, u.first_name, u.last_name, u.email
         FROM applications a 
         JOIN posts p ON a.post_id = p.id 
         JOIN accounts u ON a.applicant_id = u.unique_id
         WHERE a.employer_id = ?1"
    ) {
        Ok(stmt) => stmt,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    let applications: Result<Vec<serde_json::Value>, rusqlite::Error> = stmt
        .query_map([&auth_header], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>("id")?,
                "post_id": row.get::<_, i64>("post_id")?,
                "post_title": row.get::<_, String>("post_title")?,
                "applicant": {
                    "id": row.get::<_, String>("applicant_id")?,
                    "first_name": row.get::<_, String>("first_name")?,
                    "last_name": row.get::<_, String>("last_name")?,
                    "email": row.get::<_, String>("email")?
                },
                "status": row.get::<_, String>("status")?,
                "answers": serde_json::from_str::<serde_json::Value>(&row.get::<_, String>("answers")?).unwrap_or(serde_json::json!({})),
                "created_at": row.get::<_, String>("created_at")?,
                "updated_at": row.get::<_, String>("updated_at")?
            }))
        })
        .and_then(Iterator::collect);

    match applications {
        Ok(apps) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "applications": apps
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to fetch applications: {}", e)
        }))
    }
}

// Update application status (accept/reject)
#[put("/api/v1/applications/{id}/status")]
pub async fn update_application_status(
    req: HttpRequest,
    path: web::Path<i64>,
    req_body: web::Json<UpdateApplicationStatusRequest>
) -> impl Responder {
    let application_id = path.into_inner();
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header.to_str().unwrap_or("").replace("Bearer ", ""),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "Missing authorization header"
        }))
    };

    let conn = match rusqlite::Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    // Verify user is the employer for this application
    let employer_id: String = match conn.query_row(
        "SELECT employer_id FROM applications WHERE id = ?1",
        [&application_id],
        |row| row.get(0)
    ) {
        Ok(id) => id,
        Err(_) => return HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": "Application not found"
        }))
    };

    if employer_id != auth_header {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "error": "You can only update status for your own applications"
        }));
    }

    // Validate status
    if req_body.status != "accepted" && req_body.status != "rejected" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": "Status must be either 'accepted' or 'rejected'"
        }));
    }

    let current_time = chrono::Utc::now().to_rfc3339();

    match conn.execute(
        "UPDATE applications SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![req_body.status, current_time, application_id],
    ) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": format!("Application {} successfully", req_body.status)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to update application status: {}", e)
        }))
    }
}