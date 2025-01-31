use actix_web::{Responder, post, HttpResponse, get, web};
use serde::{Deserialize, Serialize};

use crate::users;

#[derive(Deserialize, Serialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
    first_name: String,
    last_name: String,
}

#[get("/api/v1/user")]
pub async fn get_user(req: actix_web::HttpRequest) -> impl Responder {
    // Get bearer token from Authorization header
    let auth_header = match req.headers().get("Authorization") {
        Some(header) => header.to_str().unwrap_or(""),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "error": "Missing Authorization header"
        }))
    };

    // Extract UUID from bearer token
    let uuid = auth_header.replace("Bearer ", "");

    // Get user from database using UUID
    match users::NewUser::get_by_uuid(&uuid) {
        Ok(user) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "email": user.email,
            "password": user.password,
            "unique_id": user.unique_id,
            "first_name": user.first_name,
            "last_name": user.last_name,
            "profile": user.profile
        })),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": format!("User not found: {}", e)
        }))
    }
}

#[post("/api/v1/register")]
pub async fn register_account(req_body: String) -> impl Responder {
    println!("{req_body}");
    let register_request: RegisterRequest = serde_json::from_str(&req_body).unwrap();
    // Create a new user
    let new_user = users::NewUser::new(register_request.email, register_request.password, register_request.first_name, register_request.last_name);
    // Dump the new user
    if let Err(e) = users::NewUser::dump(&new_user) {
        return HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to create user: {}", e)
        }));
    }
    // Get the user's uuid
    let uuid = new_user.unique_id;
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "uuid": uuid
    }))
}

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[post("/api/v1/auth")]
pub async fn login_account(req_body: String) -> impl Responder {
    // Log incoming request
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

    // Log attempt to find user
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

    // Verify the password
    if let Some(user) = user {
        println!("[LOG] User found, verifying password...");
        match crate::enc::verify_password(&login_request.password, &user.password) {
            Ok(true) => {
                println!("[LOG] Password verification successful for user: {}", user.unique_id);
                return HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "uuid": user.unique_id
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
