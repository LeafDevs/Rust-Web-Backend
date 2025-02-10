use actix_web::{post, get, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::posts::Post;

#[derive(Deserialize, Serialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub description: String,
    pub tags: String,
    pub documents: String,
    pub tips: String,
    pub skills: String,
    pub experience: String,
    pub jobtype: String,
    pub location: String,
    pub date: String,
    pub questions: Option<String>,
    pub company_name: String,
}

#[post("/api/v1/create_post")]
pub async fn create_post(req: HttpRequest, req_body: web::Json<CreatePostRequest>) -> impl Responder {
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
            "error": "Only employers can create job posts"
        }));
    }

    // Get current profile data
    let current_profile: String = match conn.query_row(
        "SELECT profile FROM accounts WHERE unique_id = ?1",
        [&auth_header],
        |row| row.get(0)
    ) {
        Ok(p) => p,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": "Failed to fetch current profile"
        }))
    };

    // Parse current profile
    let profile: serde_json::Value = match serde_json::from_str(&current_profile) {
        Ok(p) => p,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": "Failed to parse profile data"
        }))
    };

    // Check employer agreements
    if let Some(forms) = profile.get("forms") {
        if let Some(employer) = forms.get("employer") {
            let has_agreement = employer.get("employer_agreement").and_then(|v| v.as_bool()).unwrap_or(false);
            let has_guidelines = employer.get("job_posting_guidelines").and_then(|v| v.as_bool()).unwrap_or(false);
            let has_insurance = employer.get("insurance_certificate").and_then(|v| v.as_bool()).unwrap_or(false);
            let has_benefits = employer.get("benefits_description").and_then(|v| v.as_bool()).unwrap_or(false);

            if !has_agreement || !has_guidelines || !has_insurance || !has_benefits {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "success": false,
                    "error": "Please complete all required employer forms before posting jobs"
                }));
            }
        }
    }

    match conn.execute(
        "INSERT INTO posts (title, description, tags, documents, tips, skills, experience, jobtype, location, date, questions, company_name, employer_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        rusqlite::params![
            req_body.title,
            req_body.description,
            req_body.tags,
            req_body.documents,
            req_body.tips,
            req_body.skills,
            req_body.experience,
            req_body.jobtype,
            req_body.location,
            req_body.date,
            req_body.questions,
            req_body.company_name,
            auth_header
        ],
    ) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Post created successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to create post: {}", e)
        }))
    }
}

#[get("/api/v1/posts")]
pub async fn get_posts() -> impl Responder {
    let conn = match rusqlite::Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    let mut stmt = match conn.prepare(
        "SELECT * FROM posts ORDER BY date DESC"
    ) {
        Ok(stmt) => stmt,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    let posts = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>("id")?,
            "title": row.get::<_, String>("title")?,
            "description": row.get::<_, String>("description")?,
            "tags": row.get::<_, String>("tags")?,
            "documents": row.get::<_, String>("documents")?,
            "tips": row.get::<_, String>("tips")?,
            "skills": row.get::<_, String>("skills")?,
            "experience": row.get::<_, String>("experience")?,
            "jobtype": row.get::<_, String>("jobtype")?,
            "location": row.get::<_, String>("location")?,
            "date": row.get::<_, String>("date")?,
            "questions": row.get::<_, String>("questions")?,
            "company_name": row.get::<_, String>("company_name")?,
            "employer_id": row.get::<_, String>("employer_id")?
        }))
    });

    match posts {
        Ok(posts) => {
            let posts: Result<Vec<_>, _> = posts.collect();
            match posts {
                Ok(posts) => HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "posts": posts
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to process posts: {}", e)
                }))
            }
        },
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to fetch posts: {}", e)
        }))
    }
}
