use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: Option<i32>,
    pub title: String,
    pub description: String,
    pub tags: String,
    pub documents: String,
    pub tips: String,
    pub skills: String,
    pub experience: String,
    pub jobtype: String,
    pub location: String,
    pub date: String,
    pub questions: Option<String>
}

impl Post {
    pub fn create(post: Post) -> Result<bool> {
        let conn = Connection::open("fbla.db")?;

        conn.execute(
            "INSERT INTO postings (
                title, description, tags, documents, tips, 
                skills, experience, jobtype, location, date, questions
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11
            )",
            (
                &post.title,
                &post.description, 
                &post.tags,
                &post.documents,
                &post.tips,
                &post.skills,
                &post.experience,
                &post.jobtype,
                &post.location,
                &post.date,
                &post.questions
            ),
        )?;

        Ok(true)
    }

    pub fn get_all() -> Result<Vec<Post>> {
        let conn = Connection::open("fbla.db")?;
        let mut stmt = conn.prepare(
            "SELECT id, title, description, tags, documents, tips, 
             skills, experience, jobtype, location, date, questions 
             FROM postings"
        )?;

        let post_iter = stmt.query_map([], |row| {
            Ok(Post {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                description: row.get(2)?,
                tags: row.get(3)?,
                documents: row.get(4)?,
                tips: row.get(5)?,
                skills: row.get(6)?,
                experience: row.get(7)?,
                jobtype: row.get(8)?,
                location: row.get(9)?,
                date: row.get(10)?,
                questions: row.get(11)?
            })
        })?;

        let mut posts = Vec::new();
        for post in post_iter {
            posts.push(post?);
        }

        Ok(posts)
    }
}

