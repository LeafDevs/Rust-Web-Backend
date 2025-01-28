use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use structures::AuthRequest;
#[path = "utils/enums.rs"] mod enums;
#[path = "utils/structures.rs"] mod structures;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[post("/api/v1/auth")]
async fn auth(req_body: String) -> impl Responder {
    let a: Result<AuthRequest, serde_json::Error> = serde_json::from_str(&req_body);

    println!("Recieved Body {req_body}");

    match a {
        Ok(parsed_request) => {
            HttpResponse::Ok().body(format!("Received username: {}, password: {}", parsed_request.email, parsed_request.password))
        }
        Err(_e) => {
            HttpResponse::BadRequest().body("Invalid JSON Format")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(auth)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}


// Setup Database Connection

fn initDatabase() -> rusqlite::Result<()> {
    let conn = rusqlite::Connection::open_in_memory()?;
    

    Ok(())
}

/*
    TODO
    - Recreate the Backend Routes for the api
    - Add SQLite Support
    - Fix any vulnerabilities I find
    - Add an Encryption Method (Probs a Symmetric Encryption with rotating public keys)
    - Utils class to handle password Hashing (Argon2)



    - Database Tables


*/