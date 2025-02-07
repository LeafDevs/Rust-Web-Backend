use uuid::Uuid;

use serde::Serialize;
use serde::Deserialize;

use crate::enc;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser {
    pub email: String,
    pub password: String,
    pub unique_id: String,
    pub profile: ProfileInfo,
    pub first_name: String,
    pub last_name: String,
    pub account_type: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileInfo {
    pfp: String,
    forms: Forms,
    tasks: Tasks
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Forms {
    student: StudentForms,
    employer: EmployerForms
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudentForms {
    resume: bool,
    transcript: bool,
    agreement: bool,
    background_check: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmployerForms {
    employer_agreement: bool,
    job_posting_guidelines: bool,
    insurance_certificate: bool,
    benefits_description: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tasks {
    student: Vec<String>,
    employer: Vec<String>
}

impl NewUser {
    pub fn new(email: String, password: String, first_name: String, last_name: String, account_type: String) -> NewUser {
        let hashed_password = enc::hash_password(password.as_str());
        let uuid = Uuid::new_v4().to_string();
        
        let student_forms = StudentForms {
            resume: false,
            transcript: false,
            agreement: false,
            background_check: false
        };

        let employer_forms = EmployerForms {
            employer_agreement: false,
            job_posting_guidelines: false,
            insurance_certificate: false,
            benefits_description: false
        };

        let forms = Forms {
            student: student_forms,
            employer: employer_forms
        };

        let tasks = Tasks {
            student: vec![
                "Complete profile".to_string(),
                "Upload resume".to_string(),
                "Submit required forms".to_string()
            ],
            employer: vec![
                "Complete company profile".to_string(),
                "Submit required documentation".to_string(),
                "Post job opportunities".to_string()
            ]
        };

        let profile = ProfileInfo {
            pfp: "https://github.com/leafdevs.png".to_string(),
            forms,
            tasks
        };

        return NewUser {
            email,
            password: hashed_password,
            unique_id: uuid,
            profile,
            first_name,
            last_name,
            account_type
        }
    }

    pub fn dump(&self) -> rusqlite::Result<()> {
        let conn = rusqlite::Connection::open("fbla.db")?;

        let p = serde_json::to_string(&self.profile).unwrap();

        conn.execute(
            "INSERT INTO accounts (email, password, unique_id, profile, first_name, last_name, account_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![self.email, self.password, self.unique_id, p, self.first_name, self.last_name, self.account_type],
        )?;
    
        Ok(())
    }

    pub fn get_by_uuid(uuid: &str) -> rusqlite::Result<NewUser> {
        let conn = rusqlite::Connection::open("fbla.db")?;
        let mut stmt = conn.prepare("SELECT email, password, unique_id, profile, first_name, last_name, account_type FROM accounts WHERE unique_id = ?1")?;
        
        stmt.query_row(rusqlite::params![uuid], |row| {
            Ok(NewUser {
                email: row.get(0)?,
                password: row.get(1)?,
                unique_id: row.get(2)?,
                profile: serde_json::from_str(&row.get::<_, String>(3)?).map_err(|e| {
                    println!("[ERROR] Failed to parse profile JSON: {}", e);
                    rusqlite::Error::InvalidQuery
                })?,
                first_name: row.get(4)?,
                last_name: row.get(5)?,
                account_type: row.get(6)?
            })
        })
    }

    pub fn get_by_email(email: &str) -> rusqlite::Result<Option<NewUser>> {
        println!("[LOG] Attempting to open database connection");
        let conn = rusqlite::Connection::open("fbla.db")?;
        println!("[LOG] Preparing SQL query for email: {}", email);
        let mut stmt = conn.prepare("SELECT email, password, unique_id, profile, first_name, last_name, account_type FROM accounts WHERE email = ?1")?;
        
        println!("[LOG] Executing query for user with email: {}", email);
        let user = stmt.query_row(rusqlite::params![email], |row| {
            println!("[LOG] Processing row data for user: {}", email);
            Ok(NewUser {
                email: row.get(0)?,
                password: row.get(1)?,
                unique_id: row.get(2)?,
                profile: serde_json::from_str(&row.get::<_, String>(3)?).map_err(|e| {
                    println!("[ERROR] Failed to parse profile JSON: {}", e);
                    rusqlite::Error::InvalidQuery
                })?,
                first_name: row.get(4)?,
                last_name: row.get(5)?,
                account_type: row.get(6)?
            })
        });

        match user {
            Ok(user) => {
                println!("[LOG] Successfully retrieved user with email: {}", email);
                Ok(Some(user))
            },
            Err(e) => {
                println!("[WARN] No user found with email: {}, error: {}", email, e);
                Ok(None)
            }
        }
    }
}