use jwt_simple::reexports::rand;
use jwt_simple::reexports::rand::distributions::Alphanumeric;
use jwt_simple::reexports::rand::Rng;
use sqlx::{MySqlPool, Row};
use crate::entities::login::Login;
use crate::entities::web_session::WebSession;
use chrono;

// verify user for LOGIN
pub async fn check_user(user_login: Login, pool: &rocket::State<MySqlPool>) -> bool {
    let mut _key: String = String::new();
    _key = "".to_string();

    let user_exists = sqlx::query("SELECT * FROM users WHERE username=? AND password=?")
        .bind(user_login.username)
        .bind(user_login.password)
        .fetch_one(&**pool)
        .await
        .is_ok();

    user_exists
}

// get username by session
// TODO: edit to give all user info
pub async fn get_username_by_session(session_id: String, pool: &rocket::State<MySqlPool>) -> String {
    let mut _key: String = String::new();
    _key = "".to_string();

    let out = sqlx::query("SELECT user_name FROM web_sessions WHERE session_id=?")
        .bind(session_id)
        .fetch_one(&**pool)
        .await;

    let result: String = out.expect("cannot execute query").get("user_name");
    result

}


// verify session for LOGIN - if session does not exist run logout
#[allow(dead_code)]
pub async fn verify_session_by_user_and_session_id(username: String, session_id: String, pool: &rocket::State<MySqlPool>) -> bool {
    let mut _key: String = String::new();
    _key = "".to_string();

    let session_exists = sqlx::query("SELECT * FROM web_sessions WHERE user_name=? AND session_id=?")
        .bind(username.clone())
        .bind(session_id.clone())
        .fetch_one(&**pool)
        .await;

    return if session_exists.is_ok() {
        true
    } else {
        delete_session_from_db_by_username(username.clone().to_string(), pool).await;
        false
    }
}

pub async fn verify_session_by_session_id(session_id: String, pool: &rocket::State<MySqlPool>) -> bool {
    let mut _key: String = String::new();
    _key = "".to_string();

    let session_exists = sqlx::query("SELECT * FROM web_sessions WHERE session_id=?")
        .bind(session_id.clone())
        .fetch_one(&**pool)
        .await;

    return if session_exists.is_ok() {
        true
    } else {
        false
    }
}


// check if any sessions exist for this user
pub async fn does_a_session_exist(username: String, pool: &rocket::State<MySqlPool>) -> bool {
    let mut _key: String = String::new();
    _key = "".to_string();

    let session_exists = sqlx::query("SELECT * FROM web_sessions WHERE username=?")
        .bind(username.clone())
        .fetch_one(&**pool)
        .await
        .is_ok();

    return if session_exists {
        true
    } else {
        delete_session_from_db_by_username(username.clone().to_string(), pool).await;
        false
    }
}

// create session token
pub fn create_session_key() -> String {
    let key: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    key
}


// write created session to db for user
pub async fn write_session_to_db(username: String, session_id: String, pool: &rocket::State<MySqlPool>) {
    if session_id.len() > 0 {
        let new_session = WebSession {
            user_name: username,
            session_id,
            date_created: chrono::Local::now().to_string(),
        };

        let _insert = sqlx::query(
            "INSERT INTO web_sessions (user_name, session_id, date_created)
        VALUES (?, ?, ?)")
            .bind(new_session.user_name)
            .bind(new_session.session_id)
            .bind(new_session.date_created)
            .execute(&**pool)
            .await.unwrap();
    }
}


// remove all sessions for a user from db
pub async fn delete_session_from_db_by_username(username: String, pool: &rocket::State<MySqlPool>) {
    let _delete = sqlx::query(
        "DELETE FROM web_sessions WHERE (user_name)=?")
        .bind(username)
        .execute(&**pool)
        .await.unwrap();
}
pub async fn delete_session_from_db_by_session_id(session_id: String, pool: &rocket::State<MySqlPool>) {
    let _delete = sqlx::query(
        "DELETE FROM web_sessions WHERE (session_id)=?")
        .bind(session_id)
        .execute(&**pool)
        .await.unwrap();
}
