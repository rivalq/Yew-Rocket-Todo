#[macro_use]
extern crate rocket;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::Request;
use rocket_dyn_templates::Template;
use rusqlite::{params, Connection, Error};
use std::io;

fn get_connection() -> Result<Connection, Error> {
    Connection::open("data.sqlite")
}
fn get_hash() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    rand_string
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Todo {
    id: i32,
    title: String,
}
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct User {
    username: String,
    password: String,
    token: String,
}
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct UserAdd {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct TodoAdd {
    title: String,
}
#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
struct Token {
    token: String,
}

#[derive(Debug)]
enum ApiTokenError {
    Missing,
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Token {
    type Error = ApiTokenError;

    async fn from_request(request: &'a Request<'_>) -> Outcome<Self, Self::Error> {
        let token = request.headers().get_one("token");
        match token {
            Some(token) => {
                // check validity
                Outcome::Success(Token {
                    token: token.to_string(),
                })
            }
            None => Outcome::Failure((Status::Unauthorized, ApiTokenError::Missing)),
        }
    }
}

#[post("/register", format = "json", data = "<user>")]
fn register(user: Json<UserAdd>) {
    let con = get_connection().unwrap();
    let temp = user.into_inner();
    let token = get_hash();
    con.execute(
        "INSERT into user(username,password,token) VALUES(?1,?2,?3)",
        [temp.username, temp.password, token],
    )
    .unwrap();
}
#[post("/login", format = "json", data = "<user>")]
fn login(user: Json<UserAdd>) -> Json<Token> {
    let con = get_connection().unwrap();
    let temp = user.into_inner();
    let mut stmt = con
        .prepare("SELECT token from user where username = :username and password = :password")
        .unwrap();
    let res = stmt
        .query_map(
            &[(":username", &temp.username), (":password", &temp.password)],
            |row| Ok(Token { token: row.get(0)? }),
        )
        .unwrap();
    let mut token = Token::default();
    for i in res {
        token = i.unwrap();
    }
    Json(token)
}

#[get("/todo")]
fn list_all(token: Token) -> Result<Json<Vec<Todo>>, String> {
    let con = get_connection().unwrap();
    let temp: String = token.token;
    let mut stmt = con
        .prepare("SELECT id, title FROM todo where token = :token")
        .unwrap();
    let todos = stmt
        .query_map(&[(":token", &temp)], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
            })
        })
        .unwrap();

    let mut result: Vec<Todo> = Vec::new();
    for todo in todos {
        result.push(todo.unwrap());
    }
    Ok(Json(result))
}

#[post("/todo", format = "json", data = "<todo>")]
fn add_todo(todo: Json<TodoAdd>, token: Token) {
    let con = get_connection().unwrap();
    let temp = todo.into_inner();
    let tkn = token.token;
    con.execute(
        "INSERT INTO todo(title,token) VALUES(?1,?2)",
        params![temp.title, tkn],
    )
    .unwrap();
}

#[delete("/todo/<id>")]
fn delete_todo(id: i32, token: Token) {
    let con = get_connection().unwrap();
    let tkn = token.token;
    con.execute(
        "DELETE FROM todo where id = ?1 and token = ?2",
        params![id, tkn],
    )
    .unwrap();
}
#[put("/todo/<id>", format = "json", data = "<todo>")]
fn update_todo(todo: Json<TodoAdd>, id: i32, token: Token) {
    let con = get_connection().unwrap();
    let temp = todo.into_inner();
    con.execute(
        "UPDATE todo set title = ?1 where id = ?2 and token = ?3",
        params![temp.title, id, token.token],
    )
    .unwrap();
}

#[get("/")]
async fn index() -> io::Result<NamedFile> {
    NamedFile::open("dist/index.html").await
}
#[get("/index-42c2298212e108dd_bg.wasm")]
async fn getwasm() -> io::Result<NamedFile> {
    NamedFile::open("dist/index-42c2298212e108dd_bg.wasm").await
}
#[get("/index-42c2298212e108dd.js")]
async fn getjs() -> io::Result<NamedFile> {
    NamedFile::open("dist/index-42c2298212e108dd.js").await
}
#[get("/login")]
async fn login_route() -> io::Result<NamedFile> {
    NamedFile::open("dist/index.html").await
}

#[launch]
fn rocket() -> _ {
    {
        let db_connection = get_connection().unwrap();
        /*db_connection
            .execute("DROP TABLE  if exists user ", [])
            .unwrap();
        db_connection
            .execute("DROP TABLE  if exists todo ", [])
            .unwrap();*/

        db_connection
            .execute(
                "create table if not exists user(
            username varchar(100) primary key,
            password varchar(100) not null,
            token varchar(255) not null
        )",
                [],
            )
            .unwrap();

        db_connection
            .execute(
                "create table if not exists todo(
                id integer primary key, 
                title varchar(255) not null,
                token varchar(255) not null
            )",
                [],
            )
            .unwrap();
    }

    rocket::build()
        .mount("/", routes![index, getwasm, getjs, login_route])
        .mount(
            "/api",
            routes![
                list_all,
                add_todo,
                delete_todo,
                update_todo,
                register,
                login,
            ],
        )
}
