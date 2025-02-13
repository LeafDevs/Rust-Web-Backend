use actix_web::{post, get, web, Error, HttpResponse};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use std::fs;
use std::io::Write;
use std::path::Path;
use chrono::Utc;

#[post("/api/v1/upload")]
async fn upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    println!("Starting file upload...");
    let mut new_filename = String::new();

    // Create uploads directory if it doesn't exist
    let uploads_dir = "../uploads";
    match fs::create_dir_all(uploads_dir) {
        Ok(_) => println!("Uploads directory exists or was created at: {}", uploads_dir),
        Err(e) => {
            println!("Error creating uploads directory: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Failed to create uploads directory"
            })));
        }
    }

    while let Some(field) = payload.next().await {
        let mut field = field?;
        let content_disposition = field.content_disposition();
        let original_filename = content_disposition
            .get_filename()
            .unwrap_or("unknown_file")
            .to_string();
        
        println!("Processing file: {}", original_filename);
        
        // Get file extension from original filename
        let extension = original_filename
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_string();
            
        // Generate new filename with UUID
        new_filename = format!("{}.{}", uuid::Uuid::new_v4(), extension);
        println!("Generated new filename: {}", new_filename);
        
        let filepath = format!("./src/uploads/{}", new_filename);
        println!("Attempting to create file at absolute path: {}", filepath);
        
        match fs::File::create(&filepath) {
            Ok(mut file) => {
                let mut total_bytes = 0;
                while let Some(chunk) = field.next().await {
                    match chunk {
                        Ok(data) => {
                            match file.write_all(&data) {
                                Ok(_) => {
                                    total_bytes += data.len();
                                    println!("Wrote chunk of {} bytes", data.len());
                                },
                                Err(e) => {
                                    println!("Error writing chunk to file: {}", e);
                                    return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                                        "success": false,
                                        "error": "Failed to write file chunk"
                                    })));
                                }
                            }
                        },
                        Err(e) => {
                            println!("Error reading chunk from upload: {}", e);
                            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                                "success": false,
                                "error": "Failed to read upload chunk"
                            })));
                        }
                    }
                }
                println!("Successfully wrote {} bytes to file", total_bytes);
                
                // Verify file exists after writing
                if fs::metadata(&filepath).is_ok() {
                    println!("Verified file exists at: {}", filepath);
                } else {
                    println!("WARNING: File not found after writing to: {}", filepath);
                }
            },
            Err(e) => {
                println!("Error creating file: {}", e);
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "error": "Failed to create file"
                })));
            }
        }
    }

    println!("Upload complete. Returning success response.");
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "file_url": format!("172.20.10.8:8080/uploads/{}", new_filename)
    })))
}


#[get("/health_check")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now().timestamp()
    }))
}

#[get("/get_server_time")]
pub async fn get_server_time() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "timestamp": Utc::now().timestamp()
    }))
}


#[get("/uploads/{filename}")]
pub async fn serve_file(filename: web::Path<String>) -> HttpResponse {
    let filepath = format!("./src/uploads/{}", filename);
    
    match fs::read(&filepath) {
        Ok(file_content) => {
            // Try to guess MIME type from file extension
            let content_type = match Path::new(&filepath).extension().and_then(|e| e.to_str()) {
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("png") => "image/png",
                Some("gif") => "image/gif",
                Some("mp4") => "video/mp4",
                Some("mp3") => "audio/mpeg",
                Some("wav") => "audio/wav",
                Some("pdf") => "application/pdf",
                _ => "application/octet-stream",
            };

            HttpResponse::Ok()
                .content_type(content_type)
                .body(file_content)
        },
        Err(_) => HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": "File not found"
        }))
    }
}
