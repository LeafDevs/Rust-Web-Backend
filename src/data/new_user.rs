use uuid::Uuid;

use serde::Serialize;
use serde::Deserialize;

use crate::enc;

use chrono;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser {
    pub email: String,
    pub password: String,
    pub unique_id: String,
    pub profile: ProfileInfo,
    pub first_name: String,
    pub last_name: String,
    pub account_type: String,
    pub created_at: String,
    pub last_login: String,
    pub status: String, // "active", "inactive", "suspended"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileInfo {
    pfp: String,
    forms: Forms,
    tasks: Tasks,
    bio: String,
    contact: ContactInfo,
    preferences: UserPreferences,
    education: Option<Education>,
    work_experience: Option<WorkExperience>,
    skills: Vec<String>,
    certifications: Vec<String>
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactInfo {
    phone: String,
    address: String,
    city: String,
    state: String,
    zip: String,
    country: String,
    emergency_contact: Option<EmergencyContact>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmergencyContact {
    name: String,
    relationship: String,
    phone: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPreferences {
    notification_settings: NotificationSettings,
    privacy_settings: PrivacySettings,
    job_preferences: Option<JobPreferences>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationSettings {
    email_notifications: bool,
    push_notifications: bool,
    sms_notifications: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivacySettings {
    profile_visibility: String, // "public", "private", "connections-only"
    show_email: bool,
    show_phone: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Education {
    institution: String,
    degree: String,
    field_of_study: String,
    start_date: String,
    end_date: Option<String>,
    gpa: Option<f32>,
    achievements: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkExperience {
    company: String,
    position: String,
    location: String,
    start_date: String,
    end_date: Option<String>,
    responsibilities: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobPreferences {
    desired_position: Vec<String>,
    desired_location: Vec<String>,
    salary_range: SalaryRange,
    job_type: Vec<String>, // ["full-time", "part-time", "internship"]
    remote_preference: String // "remote", "hybrid", "on-site"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalaryRange {
    minimum: i32,
    maximum: i32,
    currency: String
}

impl NewUser {
    pub fn new(email: String, password: String, first_name: String, last_name: String, account_type: String) -> NewUser {
        let hashed_password = enc::hash_password(password.as_str());
        let uuid = Uuid::new_v4().to_string();
        let current_time = chrono::Utc::now().to_rfc3339();
        
        // Create default profile info
        let profile = ProfileInfo {
            pfp: "https://github.com/leafdevs.png".to_string(),
            forms: Forms::new(),
            tasks: Tasks::new(),
            bio: String::new(),
            contact: ContactInfo {
                phone: String::new(),
                address: String::new(),
                city: String::new(),
                state: String::new(),
                zip: String::new(),
                country: String::new(),
                emergency_contact: None
            },
            preferences: UserPreferences {
                notification_settings: NotificationSettings {
                    email_notifications: true,
                    push_notifications: true,
                    sms_notifications: false
                },
                privacy_settings: PrivacySettings {
                    profile_visibility: "public".to_string(),
                    show_email: false,
                    show_phone: false
                },
                job_preferences: None
            },
            education: None,
            work_experience: None,
            skills: Vec::new(),
            certifications: Vec::new()
        };

        NewUser {
            email,
            password: hashed_password,
            unique_id: uuid,
            profile,
            first_name,
            last_name,
            account_type,
            created_at: current_time.clone(),
            last_login: current_time,
            status: "active".to_string()
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
        let mut stmt = conn.prepare_cached("SELECT email, password, unique_id, profile, first_name, last_name, account_type FROM accounts WHERE unique_id = ?1")?;
        
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
                account_type: row.get(6)?,
                created_at: String::new(),
                last_login: String::new(),
                status: String::new()
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
                account_type: row.get(6)?,
                created_at: String::new(),
                last_login: String::new(),
                status: String::new()
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

impl Forms {
    pub fn new() -> Self {
        Forms {
            student: StudentForms {
                resume: false,
                transcript: false,
                agreement: false,
                background_check: false
            },
            employer: EmployerForms {
                employer_agreement: false,
                job_posting_guidelines: false,
                insurance_certificate: false,
                benefits_description: false
            }
        }
    }
}

impl Tasks {
    pub fn new() -> Self {
        Tasks {
            student: Vec::new(),
            employer: Vec::new()
        }
    }
}