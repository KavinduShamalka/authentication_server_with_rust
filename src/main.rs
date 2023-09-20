use error::Error::*;
use auth::{with_auth, Role};

mod auth;
mod error;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::{reject, reply, Filter, Rejection, Reply};
use std::collections::HashMap;
use std::convert::Infallible;

type WebResult<T>= std::result::Result<T, Rejection>;
type  Result<T> = std::result::Result<T, Rejection>;
type Users = Arc<HashMap<String, User>>;


//Create user struct
#[derive(Clone)]
pub struct User {
    pub uid: String,
    pub email: String,
    pub pwd: String,
    pub role: String, //user or admin
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub pwd: String
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token : String
}

#[tokio::main]
async fn main() {
    let users = Arc::new(init_users());//Help us to create defaul users
    let login_route = warp::path!("login")
        .and(warp::post())
        .and(with_users(users.clone()))
        .and(warp::body::json())
        .and_then(login_handler);

    let user_route = warp::path!("user")
        .and(with_auth(Role::User))
        .and_then(user_handler);

    let admin_route = warp::path!("admin")
        .and(with_auth(Role::Admin))
        .and_then(admin_handler);

    let routes = login_route 
        .or(user_route)
        .or(admin_route)
        .recover(error::handle_rejection);

    warp::serve(routes).run(([127.0.0.1], 8090)).await;
}

fn with_users(users: Users) -> impl Filter<Extract = (Users,), Error = Infallible> + Clone {
    warp::any().map(move || users.clone())
}

pub async fn login_handler(users: User, body: LoginRequest) -> WebResult<impl Reply>{
    match users 
        .iter()
        .find(|(_uid, users)| users.email == body.email && users.pwd == body.pwd)
    {
        Some((uid, user)) => {
            let token = auth::create_jwt(&uid, &Role::from_str(&user.role()))
                .map_err(|e| reject::custom(e))?;
            Ok(reply::json(&LoginResponse{ token }))
        },
        None => Err(reject::custom(WrongCredentialsError)),
    }
}

fn init_users() -> HashMap<String, User> {
    let mut map = HashMap::new();
    map.insert(
        String::from("1"),
        User {
            uid: String::from("1"),
            email: String::from("kavindu@gmail.com"),
            pwd: String::from("1234"),
            role: String::from("User"),
        },
    );

    map.insert(
        String::from("2"),
        User {
            uid: String::from("1"),
            email: String::from("Shamalka@gmail.com"),
            pwd: String::from("4321"),
            role: String::from("Admin"),
        },
    );

    map
}