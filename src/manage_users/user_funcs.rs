use rocket::serde::json::Json;
use rocket::State;
use sqlx::{MySqlPool};
use crate::entities::login::Login;
use crate::entities::users::User;
use crate::manage_sessions::session_funcs::{check_user, create_session_key, does_a_session_exist, write_session_to_db};
use crate::manage_users::user_funcs;

pub async fn new_user(new_user: User, pool: &rocket::State<MySqlPool>) {
    let _insert = sqlx::query(
        "INSERT INTO users (username, password, email, first_name, last_name)
        VALUES (?, ?, ?, ?, ?)")
        .bind(new_user.username)
        .bind(new_user.password)
        .bind(new_user.email)
        .bind(new_user.first_name)
        .bind(new_user.last_name)
        .execute(&**pool)
        .await.unwrap();
}

pub async fn create_new_user(data: Json<User>, pool: &State<MySqlPool>) -> bool {
    let new_user = User {
        id: rocket::serde::__private::Default::default(),
        username: data.username.to_string(),
        password: data.password.to_string(),
        email: data.email.to_string(),
        first_name: data.first_name.to_string(),
        last_name: data.last_name.to_string(),
    };
    let user_query = Login {username: new_user.clone().username, password: new_user.clone().password, ipaddress: "".to_string() };
    if check_user(user_query, pool).await {
        println!("{}", ".");
        return false
    }else {
        user_funcs::new_user(new_user, pool).await;
        return true
    }
}

pub async fn login_user(
    pool: &State<MySqlPool>,
    data: Json<Login>,
) -> String {
    let user_login = Login {
        username: data.username.to_string(),
        password: data.password.to_string(),
        ipaddress: data.ipaddress.to_string(),
    };

    let mut key = String::new();
    if check_user(user_login.clone(), pool).await {
        if !does_a_session_exist(user_login.clone().username, pool).await {
            key = create_session_key();
            println!("{}", key.clone());
            write_session_to_db(user_login.clone().username, key.clone(), pool).await;
        }
    }
    key
}
