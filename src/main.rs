use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use actix_cors::Cors;
use actix_web::http;

#[path = "data/new_user.rs"] mod users;
#[path = "utils/encrypt.rs"] mod enc;

#[path = "utils/routes/accounts.rs"] mod account_routes;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let _ = init_database();
    println!("Started RESTful API on \nPublic: https://api.leafdevs.xyz/ \nPrivate: http://127.0.0.1:8080/ ");
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .allowed_methods(vec!["GET", "POST", "OPTIONS", "DELETE"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::ACCEPT,
                        actix_web::http::header::CONTENT_TYPE
                    ])
                    .max_age(3600),
            )
            .service(hello)
            .service(echo)
            .service(account_routes::get_user)
            .service(account_routes::login_account)
            .service(account_routes::register_account)
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
            unique_id VARCHAR(255) NOT NULL,
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
    


    - Stuff I did today:
    - password hashing and database work.
    - nothing else.



    - Database Tables


*/