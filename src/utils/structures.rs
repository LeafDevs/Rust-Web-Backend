use serde::{Serialize, Deserialize};

#[path = "./enums.rs"] mod enums;
use enums::AccountType;
use enums::ExperienceLevel;
use enums::JobType;
use enums::AuthType;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileInfo {
    pfp: String,
    role: AccountType
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub authtype: String,
    pub email: String,
    pub password: String,
    pub firstname: Option<String>,
    pub lastname: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    id: i32,
    name: String,
    email: String,
    password: String,
    profile: ProfileInfo
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationQuestions {
    id: i32,
    question: String,
    answer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Posting {
    id: i32,
    title: String,
    description: String,
    tags: Vec<String>, // Array for tags, Uses Vec since no Array variable in Rust
    documents: Vec<String>, // Array for documents
    tips: Vec<String>,
    skills: Vec<String>,
    experience: ExperienceLevel,
    jobtype: JobType,
    location: String,
    date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Application {
    id: i32, // Integer ID for application
    post: i32, // references the posting structure id.
    question: String,
    answer: String,
    owner: i32,
    date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Messages {
    id: i32, // Chat Message ID (Integer)
    to: i32, // who the message is to fetch profile info from the id
    from: i32, // who the message is from fetch profile info from the id
    message: String,
    isreply: bool,
    replymessage: String,
}

/*
Structs Needed:
Account
Posting
Applications
Chat

*/


