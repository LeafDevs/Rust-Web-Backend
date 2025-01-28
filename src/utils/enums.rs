use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Serialize, Deserialize)]
pub enum AccountType {
    Employer,
    Administrator,
    Student
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ExperienceLevel {
    Easy, // easy
    Intermediate, // medium
    Advanced, // hard
}
#[derive(Debug, Serialize, Deserialize)]
pub enum JobType {
    FullTime,
    PartTime
}
#[derive(Debug, Serialize, Deserialize)]
pub enum AuthType {
    Login,
    Register
}