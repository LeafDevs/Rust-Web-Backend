use rusqlite::Connection;


pub fn run_sql_command(command: &str, params: &[String]) -> rusqlite::Result<()> {

    let conn = rusqlite::Connection::open("fbla.db")?;
    let mut stmt = conn.prepare(command)?;
    stmt.execute(rusqlite::params_from_iter(params))?;


    Ok(())
}

pub fn create_account(email: &str, password: &str, name: &str) -> rusqlite::Result<()> {
    let parts: Vec<&str> = name.split_whitespace().collect();
    let first_name = parts.get(0).unwrap_or(&"");
    let last_name = parts.get(1..).map(|s| s.join(" ")).unwrap_or_default();

    let conn = rusqlite::Connection::open("fbla.db")?;
    conn.execute(
        "INSERT INTO accounts (email, password, first_name, last_name) VALUES (?, ?, ?, ?)",
        rusqlite::params![email, password, first_name, last_name],
    )?;

    Ok(())
}

pub fn check_account(email: &str, password: &str) -> rusqlite::Result<bool> {
    let conn = match Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => {
            println!("Database connection failed: {}", e);
            return Err(e);
        }
    };

    let mut stmt = match conn.prepare("SELECT COUNT(*) FROM accounts WHERE email = ?1 AND password = ?2") {
        Ok(stmt) => stmt,
        Err(e) => {
            println!("Failed to prepare statement: {}", e);
            return Err(e);
        }
    };

    let count: i32 = match stmt.query_row(rusqlite::params![email, password], |row| row.get(0)) {
        Ok(count) => count,
        Err(e) => {
            println!("Failed to execute query: {}", e);
            return Err(e);
        }
    };

    if count > 0 {
        println!("Account found for email: {}", email);
        Ok(true)
    } else {
        println!("No matching account for email: {}", email);
        Ok(false)
    }
}



pub fn create_posting(
    title: &str,
    description: &str,
    tags: &str,
    documents: &str,
    skills: &str,
    experience: &str,
    jobtype: &str,
    location: &str,
    tips: &str,
) -> rusqlite::Result<()> {
    let conn = match Connection::open("fbla.db") {
        Ok(conn) => conn,
        Err(e) => {
            println!("Database connection failed: {}", e);
            return Err(e);
        }
    };

    let result = conn.execute(
        "INSERT INTO postings (title, description, tags, documents, skills, experience, jobtype, location, tips) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![title, description, tags, documents, skills, experience, jobtype, location, tips],
    );

    match result {
        Ok(rows) => {
            println!("Successfully inserted posting: {} ({} row affected)", title, rows);
            Ok(())
        }
        Err(e) => {
            println!("Failed to insert posting: {} - {}", title, e);
            Err(e)
        }
    }
}
