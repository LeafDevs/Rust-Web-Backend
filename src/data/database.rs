use rusqlite::Connection;
use rusqlite::params;
use serde_json;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> rusqlite::Result<Database> {
        let conn = Connection::open("fbla.db")?;
        Ok(Database { conn })
    }

    pub fn fetch_user(&self, uuid: &str) -> rusqlite::Result<Option<NewUser>> {
        let mut stmt = self.conn.prepare("SELECT * FROM users WHERE unique_id = ?1")?;
        let user = stmt.query_row(params![uuid], |row| {
            Ok(NewUser {
                email: row.get(0)?,
                password: row.get(1)?,
                unique_id: row.get(2)?,
                profile: serde_json::from_str(&row.get::<_, String>(3)?)?,
                first_name: row.get(4)?,
                last_name: row.get(5)?,
            })
        });
        user.ok()
    }

    pub fn insert_user(&self, user: &NewUser) -> rusqlite::Result<()> {
        let p = serde_json::to_string(&user.profile).unwrap();
        self.conn.execute(
            "INSERT INTO users (email, password, unique_id, profile, first_name, last_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![user.email, user.password, user.unique_id, p, user.first_name, user.last_name],
        )?;
        Ok(())
    }
}
