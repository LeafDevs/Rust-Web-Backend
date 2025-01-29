use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use structures::AuthRequest;
use dotenv::dotenv;
#[path = "utils/enums.rs"] mod enums;
#[path = "utils/structures.rs"] mod structures;
#[path = "utils/database.rs"] mod database;
#[path = "utils/encrypt.rs"] mod enc;

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
            let success = database::check_account(&parsed_request.email, &parsed_request.password).unwrap_or(false);
            HttpResponse::Ok().body(format!(r#"{{"success": {}}}"#, success))
        }
        Err(_e) => {
            HttpResponse::BadRequest().body("Invalid JSON Format")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let _ = init_database();
    let _ = database::create_account("email", "passowrd", "name name");
    let hash = enc::hash_password("test_PasswoRd123@@#");
    println!("test_PasswoRd123@@#");
    println!("{}", hash);
    println!("Created Database");
    // let _ = database::create_posting("Retail Cashier", "Work as a cashier while also working the queue", "123", "123", "123", "1234", "jobtype", "location", "none");
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

fn init_database() -> rusqlite::Result<()> {
    let conn = rusqlite::Connection::open("fbla.db")?;

    conn.execute(
        "create table if not exists accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            first_name VARCHAR(255) NULL,
            last_name VARCHAR(255) NULL,
            email VARCHAR(255) NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            password VARCHAR(255) NOT NULL,
            profile TEXT NULL DEFAULT '{\"pfp\": \"https://github.com/leafdevs.png\", \"role\": \"Student\"}' )",
        [],
    )?;

    conn.execute(
        "create table if not exists postings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            tags TEXT NOT NULL,
            documents TEXT NOT NULL,
            tips TEXT NOT NULL,
            skills TEXT NOT NULL,
            experience TEXT NOT NULL,
            jobtype TEXT NOT NULL,
            location TEXT NOT NULL,
            date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
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