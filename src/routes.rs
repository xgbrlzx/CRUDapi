use axum::
{
    extract::{ Path, State },
    routing::{ get, post, put, delete },
    http::{HeaderMap, header::ACCEPT},
    response::{ IntoResponse, Html, Response },
    Json,
    Router
};
use sqlx::{ Row, Column, FromRow };

use crate::utils::{ fetch, sanitize_user_input, execute, gen_status_response, ArcDB, User };


pub fn map_routes(arc_db: ArcDB) -> Router
{
    return Router::new()
        .route("/", get(root))
        .route("/hello/:name", get(hello))
        .route("/users", get(get_all_users))
        .route("/users", post(create_user)).with_state(arc_db.clone())
        .route("/users/:login", get(get_user))
        .route("/users/:login", put(update_user)).with_state(arc_db.clone())
        .route("/users/:login", delete(delete_user)).with_state(arc_db.clone())
    ;
}

async fn get_user(State(db): State<ArcDB>, headers: HeaderMap, Path(mut user): Path<String>) -> Response
{
    sanitize_user_input(&mut user);
    if let Ok(rows) = fetch(&format!("select * from usuario where login='{user}'"), &*db).await
    {
        if rows.is_empty() { gen_status_response("user not found", "", &headers) }
        else
        {
            if headers.contains_key(ACCEPT) && headers[ACCEPT].to_str().unwrap_or("").contains("json")
            {
                if let Ok(user) = User::from_row(&rows[0]) { Json(user).into_response() }
                else { gen_status_response("could not get user", "internal server error", &headers) }
            }
            else
            {
                let mut string = String::new();
                for col in rows[0].columns()
                {
                    let val = rows[0].try_get(col.name()).unwrap_or("|null|");
                    string = string + col.name() + ": " + val + "<br>";
                }
                
                Html
                (
                    format!
                    (
                        "<!DOCTYPE html>
                            <html>
                                <h1> {string} </h1>
                            </html>
                        "
                    )
                ).into_response()
            }
        }
    }
    else { gen_status_response("could not retrieve user from database", "internal server error", &headers) }
}

async fn get_all_users(State(db): State<ArcDB>, headers: HeaderMap) -> Response
{
    if let Ok(rows) = fetch(&format!("select * from usuario"), &*db).await
    {
        if rows.is_empty() { gen_status_response("user not found", "", &headers) }
        else
        {
            if headers.contains_key(ACCEPT) && headers[ACCEPT].to_str().unwrap_or("").contains("json")
            {
                let mut users = Vec::new();
                for row in rows
                {
                    if let Ok(user) = User::from_row(&row) { users.push(user) }
                }
                Json(users).into_response()
            }
            else
            {
                let mut string = String::new();
                for row in rows
                {
                    for col in row.columns()
                    {
                        let val = row.try_get(col.name()).unwrap_or("|null|");
                        string = string + col.name() + ": " + val + "<br>";
                    }
                    string+="<br>"
                }
                Html
                (
                    format!
                    (
                        "<!DOCTYPE html>
                            <html>
                                <h1> {string} </h1>
                            </html>
                        "
                    )
                ).into_response()
            }
        }
    }
    else { gen_status_response("could not retrieve users from database", "internal server error", &headers) }
}

async fn create_user(State(db): State<ArcDB>, headers: HeaderMap, Json(mut payload): Json<serde_json::Value>) -> Response
{
    let data = (payload["nome"].take(), payload["login"].take(), payload["senha"].take());
    match data
    {
        (serde_json::Value::String(mut nome), serde_json::Value::String(mut login), serde_json::Value::String(mut senha)) => // valid json body
        {
            sanitize_user_input(&mut nome);
            sanitize_user_input(&mut login);
            sanitize_user_input(&mut senha);

            if nome.len() > 50 || login.len() > 30 || senha.len() > 30
            {
                gen_status_response("could not create user", "'nome' must not be longer than 50 characters, 'login' and 'senha' must not be longer than 30 characters", &headers)
            }
            else if let Ok(rows) = fetch(&format!("select * from usuario where login='{login}'"), &*db).await
            {
                if rows.is_empty()
                {
                    if let Ok(_) = execute(&format!("insert into usuario (nome, login, senha) values ('{nome}', '{login}', '{senha}')"), &*db).await
                    {
                        gen_status_response("user created successfully", "", &headers)
                    }
                    else { gen_status_response("could not create user", "intenal server error", &headers) }
                }
                else // login already registered
                {
                    gen_status_response("could not create user", "login already in use, please try a different one", &headers)
                }
            }
            else { gen_status_response("could not create user", "intenal server error", &headers) }
        }
        _ => { gen_status_response("could not create user", "expected json with fields 'nome', 'login' and 'senha' with a string value", &headers) }
    }
}


async fn root() -> impl IntoResponse
{
    Html
    (
        "<!DOCTYPE html>
        <html>
            <h1> Hello, World! </h1>
        </html>"
    )
}
/*
async fn update_user() -> impl IntoResponse
{
    let status_code;
    let html_response;
    
    (status_code, html_response)
}
*/

async fn delete_user(State(db): State<ArcDB>, headers: HeaderMap, Path(login): Path<String>) -> Response
{
    if let Ok(res) = execute(&format!("delete from usuario where login='{}'", login), &*db).await
    {
        if res.rows_affected() == 0
        {
            gen_status_response("could not delete user", "user not found", &headers)
        }
        else { gen_status_response("user deleted successfully", "", &headers) }
    }
    else { gen_status_response("could not delete user", "internal server error", &headers) }
}

async fn update_user(State(db): State<ArcDB>, headers: HeaderMap, Path(login): Path<String>, Json(mut payload): Json<serde_json::Value>) -> Response
{
    let data = (payload["nome"].take(), payload["login"].take(), payload["senha"].take());
    match data
    {
        (serde_json::Value::String(mut new_nome), serde_json::Value::String(mut new_login), serde_json::Value::String(mut new_senha)) => // valid json body
        {
            sanitize_user_input(&mut new_nome);
            sanitize_user_input(&mut new_login);
            sanitize_user_input(&mut new_senha);

            if new_nome.len() > 50 || new_login.len() > 30 || new_senha.len() > 30
            {
                gen_status_response("could not update user", "'nome' must not be longer than 50 characters, 'login' and 'senha' must not be longer than 30 characters", &headers)
            }
            else if let Ok(res) = execute(&format!("update usuario set nome = '{new_nome}', login = '{new_login}', senha = '{new_senha} where login = '{login}'"), &*db).await
            {
                if res.rows_affected() == 0
                {
                    gen_status_response("could not update user", "user not found", &headers)
                }
                else { gen_status_response("user updated succesfully", "", &headers) }
            }
            else { gen_status_response("could not update user", "intenal server error", &headers) }
        }
        _ => { gen_status_response("could not update user", "expected json with fields 'nome', 'login' and 'senha' with a string value", &headers) }    
    }
}

async fn hello(Path(mut name): Path<String>, headers: HeaderMap) -> impl IntoResponse
{
    println!("|{:?}|", headers);
    if headers.contains_key(ACCEPT) && headers[ACCEPT].to_str().unwrap_or("").contains("json")
    {
        return Json::from(format!("{{\"hello\": \"{name}\"}}")).into_response();
    }
    sanitize_user_input(&mut name);
    Html
    (
        format!
        (
            "<!DOCTYPE html>
                <html>
                    <h1> Hello, {name} </h1>
                </html>
            "
        )
    ).into_response()
}