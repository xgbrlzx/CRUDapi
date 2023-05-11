mod utils;
mod routes;
use routes::map_routes;


#[tokio::main]
async fn main()
{
    // database connection url
    let db_url = "mysql://127.0.0.1:3306/api_mes";

    // address to listen to api requests
    let api_addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));

    // mysql db connection, shareable between threads
    let arc_db_pool = std::sync::Arc::new(sqlx::MySqlPool::connect(db_url).await.expect("could not connect to db"));

    // map routes
    let api = map_routes(arc_db_pool);

    // launch the server
    axum::Server::bind(&api_addr).serve(api.into_make_service()).await.expect("could not launch server at given address");
}
