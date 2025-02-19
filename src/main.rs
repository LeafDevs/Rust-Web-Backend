use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use actix_cors::Cors;
use actix_web::http;

#[path = "data/new_user.rs"] mod users;
#[path = "utils/encrypt.rs"] mod enc;
#[path = "data/posts.rs"] mod posts;

#[path = "utils/routes/messages.rs"] mod message_routes;
#[path = "utils/routes/accounts.rs"] mod account_routes;
#[path = "utils/routes/posts.rs"] mod post_routes;
#[path = "utils/routes/misc.rs"] mod misc_routes;

#[path = "utils/routes/applications.rs"] mod application_routes;

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
                    .allowed_origin("http://127.0.0.1:5173")
                    .allowed_origin("https://jobs.leafdevs.xyz")
                    .allowed_origin("https://api.leafdevs.xyz")
                    .allowed_origin("http://127.0.0.1:5173")
                    .allowed_methods(vec!["GET", "POST", "OPTIONS", "DELETE", "PUT"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::ACCEPT,
                        actix_web::http::header::CONTENT_TYPE
                    ])
                    .max_age(3600),
            )
            .service(hello)
            .service(echo)

            // Account Routes
            .service(account_routes::get_user)
            .service(account_routes::login_account)
            .service(account_routes::register_account)
            .service(account_routes::update_employer_agreements)
            .service(account_routes::get_total_employers)
            .service(account_routes::get_total_users)
            .service(account_routes::get_all_users_without_private_information_leaked)
            .service(message_routes::get_messages)
            .service(message_routes::get_conversations)
            .service(message_routes::send_message)

            // Post Routes
            .service(post_routes::get_posts)
            .service(post_routes::create_post)
            .service(post_routes::accept_post)
            .service(post_routes::reject_post)
            .service(post_routes::get_pending_posts)
            .service(post_routes::get_my_posts)
            .service(post_routes::delete_post)
            .service(post_routes::update_post)

            // Application Routes
            .service(application_routes::create_application)
            .service(application_routes::get_received_applications)
            .service(application_routes::get_submitted_applications)
            .service(application_routes::update_application_status)
            

            // Misc Routes

            .service(misc_routes::upload)
            .service(misc_routes::health_check)
            .service(misc_routes::get_server_time)
            .service(misc_routes::serve_file)

            // Post Routes

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
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sender_id INTEGER NOT NULL,
            receiver_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            read BOOLEAN DEFAULT FALSE,
            message_type TEXT,
            file_url TEXT,
            FOREIGN KEY (sender_id) REFERENCES accounts(id),
            FOREIGN KEY (receiver_id) REFERENCES accounts(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email VARCHAR(255) NOT NULL UNIQUE,
            password VARCHAR(255) NOT NULL,
            unique_id VARCHAR(255) NOT NULL UNIQUE,
            first_name VARCHAR(255) NOT NULL,
            last_name VARCHAR(255) NOT NULL,
            account_type VARCHAR(50) NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_login DATETIME,
            status VARCHAR(50) NOT NULL DEFAULT 'active',
            profile TEXT NOT NULL,
            CONSTRAINT email_unique UNIQUE (email),
            CONSTRAINT uuid_unique UNIQUE (unique_id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
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
            date TEXT NOT NULL,
            questions TEXT NOT NULL,
            company_name TEXT NOT NULL,
            employer_id TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'Pending' CHECK (status IN ('Accepted', 'Pending')),
            FOREIGN KEY (employer_id) REFERENCES accounts (unique_id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS applications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            post_id INTEGER NOT NULL,
            applicant_id TEXT NOT NULL,
            employer_id TEXT NOT NULL,
            status TEXT NOT NULL,
            answers TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (post_id) REFERENCES posts (id),
            FOREIGN KEY (applicant_id) REFERENCES accounts (unique_id),
            FOREIGN KEY (employer_id) REFERENCES accounts (unique_id)
        )",
        [],
    )?;
    Ok(())
}

/*
    TODO
    ✓ Recreate the Backend Routes for the api
    ✓ Add SQLite Support
    - Fix any vulnerabilities I find
    ✓ Add an Encryption Method (Probs a Symmetric Encryption with rotating public keys)
    ✓ Utils class to handle password Hashing (Argon2)
    ✓ Add a method to handle the file uploads
    ✓ Messaging Feature
    ✓ Application Feature
    ✓ Admin Related Features

*/