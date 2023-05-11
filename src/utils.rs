use std::sync::Arc;
use axum::{Json, response::{IntoResponse, Response}, http::{HeaderMap, StatusCode, header::ACCEPT}};
use serde::Serialize;
use sqlx::{ IntoArguments, Executor, Database, Pool, MySql, database, FromRow };


// wraps the chosen database pool in an arc so it can be used in multiple threads, new type to make it shorter and abstract away the concrete DB
pub type ArcDB = Arc<Pool<MySql>>;

// sanitizes user input to avoid sql injections
pub fn sanitize_user_input(input: &mut String)
{
    let mut indexes = Vec::new();
    for (index, char) in input.char_indices()
    {
        if char == '\'' { indexes.push(index); }
    }

    indexes.reverse();
    for index in indexes { input.insert(index, '\\'); }
}

// wraps the specific database, row and error types in generic abstractions
pub async fn fetch<'a, E, DB>(query: &'a str, executor: E) -> Result<Vec<DB::Row>, impl std::fmt::Display + std::fmt::Debug + Send + Sync + 'static>
where E: Executor<'a, Database = DB>,
      DB: Database,
      <DB as database::HasArguments<'a>>::Arguments: IntoArguments<'a, DB>
{
    match sqlx::query(query).fetch_all(executor).await
    {
        Ok(rows) => { Ok(rows) }
        Err(e) => { Err(e) }
    }
}

// wraps the specific database, row and error types in generic abstractions
pub async fn execute<'a, E, DB>(query: &'a str, executor: E) -> Result<DB::QueryResult, impl std::fmt::Display + std::fmt::Debug + Send + Sync + 'static>
where E: Executor<'a, Database = DB>,
      DB: Database,
      <DB as database::HasArguments<'a>>::Arguments: IntoArguments<'a, DB>
{
    match sqlx::query(query).execute(executor).await
    {
        Ok(result) => { Ok(result) }
        Err(e) => { Err(e) }
    }
}

pub fn html_status_page(status_msg: &str, extra_info: &str) -> axum::response::Html<String>
{
    axum::response::Html
    (
        format!
        (
            "<!DOCTYPE html>
                <html>
                    <h1> {status_msg} </h1>
                    <p1> {extra_info} </p1>
                </html>
            "
        )
    )
}

#[derive(Debug, Serialize)]
pub struct StatusData
{
    status_msg: String,
    error: Option<String>
}

#[derive(Debug, Serialize, FromRow)]
pub struct User
{
    nome: String,
    login: String,
    senha: String
}

pub fn json_status_response(status_msg: String, error: Option<String>) -> Json<StatusData>
{
    Json::from(StatusData { status_msg: status_msg, error: error } )

}
pub fn gen_status_response(status_msg: &str, extra_info: &str, headers: &HeaderMap) -> Response
{
    let status_code = { if extra_info == "internal server error" { StatusCode::INTERNAL_SERVER_ERROR } else { StatusCode::BAD_REQUEST } };
    let client_expects_json = headers.contains_key(ACCEPT) && headers[ACCEPT].to_str().unwrap_or("").contains("json");
    let response;
    if client_expects_json { response = json_status_response(status_msg.to_string(), if extra_info == "" { None } else { Some(extra_info.to_string()) }).into_response() }
    else{ response = html_status_page(status_msg, extra_info).into_response() }

    (status_code, response).into_response()
}