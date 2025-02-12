use actix_web::{Responder, post, HttpResponse, get, web};
use serde::{Deserialize, Serialize};

use crate::users;

#[derive(Deserialize, Serialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
    first_name: String,
    last_name: String,
    account_type: String
}

#[derive(Deserialize)]
pub struct UpdateEmployerAgreementsRequest {
    employer_agreement: bool,
    job_posting_guidelines: bool,
    insurance_certificate: bool,
    benefits_description: bool
}

#[get("/api/v1/total_users")]
pub async fn get_total_users() -> impl Responder {
    let conn = match rusqlite::Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    let total: i64 = match conn.query_row(
        "SELECT COUNT(*) FROM accounts",
        [],
        |row| row.get(0)
    ) {
        Ok(count) => count,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to count users: {}", e)
        }))
    };

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "total_users": total
    }))
}

#[get("/api/v1/total_employers")]
pub async fn get_total_employers() -> impl Responder {
    let conn = match rusqlite::Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    let total: i64 = match conn.query_row(
        "SELECT COUNT(*) FROM accounts WHERE account_type = 'employer'",
        [],
        |row| row.get(0)
    ) {
        Ok(count) => count,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to count employers: {}", e)
        }))
    };

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "total_employers": total
    }))
}

#[get("/api/v1/user")]
pub async fn get_user(req: actix_web::HttpRequest) -> impl Responder {
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header.to_str().unwrap_or(""),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "Missing Authorization header"
        }))
    };

    let uuid = auth_header.replace("Bearer ", "");

    match users::NewUser::get_by_uuid(&uuid) {
        Ok(user) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "email": user.email,
            "unique_id": user.unique_id,
            "first_name": user.first_name,
            "last_name": user.last_name,
            "profile": user.profile,
            "forms": {
                "student": {
                    "resume": false,
                    "transcript": false,
                    "agreement": false,
                    "background_check": false
                },
                "employer": {
                    "employer_agreement": false,
                    "job_posting_guidelines": false,
                    "insurance_certificate": false,
                    "benefits_description": false
                }
            },
            "tasks": {
                "student": [
                    "Complete profile",
                    "Upload resume",
                    "Submit required forms"
                ],
                "employer": [
                    "Complete company profile",
                    "Submit required documentation",
                    "Post job opportunities"
                ]
            }
        })),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": format!("User not found: {}", e)
        }))
    }
}

#[post("/api/v1/register")]
pub async fn register_account(req_body: String) -> impl Responder {
    match serde_json::from_str::<RegisterRequest>(&req_body) {
        Ok(register_request) => {
            let new_user = users::NewUser::new(
                register_request.email,
                register_request.password,
                register_request.first_name,
                register_request.last_name,
                register_request.account_type.clone()
            );
            if let Err(e) = users::NewUser::dump(&new_user) {
                return HttpResponse::Ok().json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to create user: {}", e)
                }));
            }
            let uuid = new_user.unique_id;
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "uuid": uuid,
                "account_type": register_request.account_type
            }))
        },
        Err(e) => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": format!("Invalid request format: {}", e)
            }))
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[post("/api/v1/auth")]
pub async fn login_account(req_body: String) -> impl Responder {
    println!("[LOG] Login request received: {}", req_body);

    let login_request: LoginRequest = match serde_json::from_str(&req_body) {
        Ok(request) => request,
        Err(e) => {
            println!("[ERROR] Failed to parse login request: {}", e);
            return HttpResponse::Ok().json(serde_json::json!({
                "success": false,
                "error": "Invalid request format"
            }));
        }
    };

    println!("[LOG] Attempting to find user with email: {}", login_request.email);

    let user = match users::NewUser::get_by_email(&login_request.email) {
        Ok(user) => {
            println!("[LOG] User query successful");
            user
        },
        Err(e) => {
            println!("[ERROR] Database error while fetching user: {}", e);
            return HttpResponse::Ok().json(serde_json::json!({
                "success": false,
                "error": "Database error"
            }))
        }
    };

    if let Some(user) = user {
        println!("[LOG] User found, verifying password...");
        match crate::enc::verify_password(&login_request.password, &user.password) {
            Ok(true) => {
                println!("[LOG] Password verification successful for user: {}", user.unique_id);
                return HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "uuid": user.unique_id,
                    "account_type": user.account_type
                }));
            },
            Ok(false) => {
                println!("[LOG] Password verification failed for user: {}", user.unique_id);
            },
            Err(e) => {
                println!("[ERROR] Password verification error: {}", e);
            }
        }
    } else {
        println!("[LOG] No user found with email: {}", login_request.email);
    }

    println!("[LOG] Login attempt failed for email: {}", login_request.email);
    HttpResponse::Ok().json(serde_json::json!({
        "success": false,
        "error": "Invalid email or password"
    }))
}

#[post("/api/v1/employer/agreements")]
pub async fn update_employer_agreements(
    req: actix_web::HttpRequest,
    agreements: web::Json<UpdateEmployerAgreementsRequest>
) -> impl Responder {
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
            "error": "Only employers can update employer agreements"
        }));
    }

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

    let mut profile: serde_json::Value = match serde_json::from_str(&current_profile) {
        Ok(p) => p,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": "Failed to parse profile data"
        }))
    };

    if let Some(forms) = profile.get_mut("forms") {
        if let Some(employer) = forms.get_mut("employer") {
            employer["employer_agreement"] = serde_json::json!(agreements.employer_agreement);
            employer["job_posting_guidelines"] = serde_json::json!(agreements.job_posting_guidelines);
            employer["insurance_certificate"] = serde_json::json!(agreements.insurance_certificate);
            employer["benefits_description"] = serde_json::json!(agreements.benefits_description);
        }
    }

    match conn.execute(
        "UPDATE accounts SET profile = ?1 WHERE unique_id = ?2",
        [&profile.to_string(), &auth_header]
    ) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Employer agreements updated successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to update agreements: {}", e)
        }))
    }
}

#[get("/api/v1/users")]
pub async fn get_all_users_without_private_information_leaked() -> impl Responder {
    let conn = match rusqlite::Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    };

    let mut stmt = match conn.prepare(
        "SELECT unique_id, first_name, last_name, profile, account_type FROM accounts"
    ) {
        Ok(stmt) => stmt,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to prepare statement: {}", e)
        }))
    };

    let users_iter = stmt.query_map([], |row| {
        let profile_str: String = row.get(3)?;
        let profile: serde_json::Value = serde_json::from_str(&profile_str).unwrap_or_default();

        // Extract profile picture from the profile JSON
        let profile_picture = profile.get("pfp")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "first_name": row.get::<_, String>(1)?,
            "last_name": row.get::<_, String>(2)?,
            "pfp": profile_picture,
            "account_type": row.get::<_, String>(4)?
        }))
    });

    let users: Vec<serde_json::Value> = match users_iter {
        Ok(iter) => iter.filter_map(Result::ok).collect(),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to fetch users: {}", e)
        }))
    };

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "users": users
    }))
}