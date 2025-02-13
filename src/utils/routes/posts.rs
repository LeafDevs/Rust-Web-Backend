use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use rusqlite::Connection;
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
        "INSERT INTO posts (title, description, tags, documents, tips, skills, experience, jobtype, location, date, questions, company_name, employer_id, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
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
            auth_header,
            "Pending"
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
        "SELECT * FROM posts WHERE status = 'Accepted' ORDER BY date DESC"
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

#[get("/api/v1/pending_posts")]
pub async fn get_pending_posts(req: HttpRequest) -> impl Responder {
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
            "error": format!("Database connection failed: {}", e)
        }))
    };

    // Verify user is admin
    let is_admin: bool = match conn.query_row(
        "SELECT account_type FROM accounts WHERE unique_id = ?",
        [&auth_header],
        |row| {
            let account_type: String = row.get(0)?;
            Ok(account_type == "administrator")
        }
    ) {
        Ok(is_admin) => is_admin,
        Err(_) => return HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "Unauthorized access"
        }))
    };

    if !is_admin {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "error": "Only administrators can view pending posts"
        }));
    }

    let mut stmt = match conn.prepare(
        "SELECT * FROM posts WHERE status = 'Pending' ORDER BY date DESC"
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


#[get("/api/v1/my_posts")]
pub async fn get_my_posts(req: HttpRequest) -> impl Responder {
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
            "error": format!("Database connection failed: {}", e)
        }))
    };

    let mut stmt = match conn.prepare("SELECT * FROM posts WHERE employer_id = ?") {
        Ok(stmt) => stmt,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false, 
            "error": format!("Failed to prepare statement: {}", e)
        }))
    };

    let posts = stmt.query_map([&auth_header], |row| {
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
            "company_name": row.get::<_, String>("company_name")?
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

#[delete("/api/v1/posts/{id}")]
pub async fn delete_post(req: HttpRequest) -> impl Responder {
    let id = req.match_info().get("id").unwrap();
    println!("Attempting to delete post with ID: {}", id);

    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header.to_str().unwrap_or("").replace("Bearer ", ""),
        None => {
            println!("Delete post failed: No authorization header provided");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "success": false,
                "error": "No authorization header provided"
            }))
        }
    };

    let conn = match Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => {
            println!("Delete post failed: Database connection error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Database connection error: {}", e)
            }))
        }
    };

    // First verify the post belongs to this employer
    let post_owner = match conn.query_row(
        "SELECT employer_id FROM posts WHERE id = ?",
        [id],
        |row| row.get::<_, String>(0)
    ) {
        Ok(employer_id) => employer_id,
        Err(e) => {
            println!("Delete post failed: Could not verify post ownership: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false, 
                "error": format!("Failed to verify post ownership: {}", e)
            }))
        }
    };

    if post_owner != auth_header {
        println!("Delete post failed: Unauthorized attempt to delete post {} by user {}", id, auth_header);
        return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "error": "You do not have permission to delete this post"
        }));
    }

    // First delete any related records in child tables
    match conn.execute("DELETE FROM applications WHERE post_id = ?", [id]) {
        Ok(_) => (),
        Err(e) => {
            println!("Delete post failed: Could not delete related applications: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to delete related applications: {}", e)
            }))
        }
    }

    // Then delete the post
    match conn.execute("DELETE FROM posts WHERE id = ?", [id]) {
        Ok(_) => {
            println!("Successfully deleted post with ID: {}", id);
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("Post with ID {} deleted successfully", id)
            }))
        },
        Err(e) => {
            println!("Delete post failed: Database error while deleting post {}: {}", id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to delete post: {}", e)
            }))
        }
    }
}

#[put("/api/v1/posts/{id}")]
pub async fn update_post(req: HttpRequest, body: web::Json<Post>) -> impl Responder {
    let id = match req.match_info().get("id") {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": "Missing post ID"
        }))
    };

    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header.to_str().unwrap_or("").replace("Bearer ", ""),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "No authorization header provided"
        }))
    };

    let conn = match rusqlite::Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Database connection failed: {}", e)
        }))
    };

    // Verify post ownership
    let post_owner: String = match conn.query_row(
        "SELECT employer_id FROM posts WHERE id = ?",
        [id],
        |row| row.get(0)
    ) {
        Ok(owner) => owner,
        Err(e) => return HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": format!("Post not found: {}", e)
        }))
    };

    if post_owner != auth_header {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "success": false,
            "error": "You do not have permission to update this post"
        }));
    }

    let post = body.into_inner();
    match conn.execute(
        "UPDATE posts SET 
            title = ?,
            description = ?,
            tags = ?,
            documents = ?,
            tips = ?,
            skills = ?,
            experience = ?,
            jobtype = ?,
            location = ?,
            date = ?,
            questions = ?
        WHERE id = ?",
        rusqlite::params![
            post.title,
            post.description,
            serde_json::to_string(&post.tags).unwrap(),
            post.documents,
            serde_json::to_string(&post.tips).unwrap(),
            serde_json::to_string(&post.skills).unwrap(),
            post.experience,
            post.jobtype,
            post.location,
            post.date,
            serde_json::to_string(&post.questions).unwrap(),
            id
        ],
    ) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Post updated successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to update post: {}", e)
        }))
    }
}