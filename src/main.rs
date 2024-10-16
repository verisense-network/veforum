use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use tower_http::trace::TraceLayer;

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

// use sqlx
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

// Define your Rust structure
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    email: String,
}

// Assume this function gets your database connection URL.
fn get_database_url() -> String {
    "postgres://username:password@localhost/database".to_string()
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // Create a connection pool
    let pool = PgPool::connect(&get_database_url()).await?;

    // Define your query and map directly to the User struct
    let users: Vec<User> = sqlx::query_as("SELECT id, username, email FROM users")
        .fetch_all(&pool)
        .await?;

    // Here you can now use `users` which is a vector of `User` structs directly.
    println!("Users: {:#?}", users);

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=localhost user=postgres dbname=postgres password=123456",
        NoTls,
    )
    .unwrap();

    let pool = Pool::builder().build(manager).await.unwrap();

    let app = Router::new()
        .route("/", get(handler))
        .route("/query_from_db", get(query_from_db))
        .layer(TraceLayer::new_for_http())
        .fallback(handler_404)
        .with_state(pool);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

async fn query_from_db(State(pool): State<ConnectionPool>) -> Result<String, (StatusCode, String)> {
    tracing::debug!("get db conn {:?}", pool);
    let conn = pool.get().await.map_err(internal_error)?;

    tracing::debug!("query_from_db: 1");
    let row = conn
        .query_one("select 1 + 1", &[])
        .await
        .map_err(internal_error)?;
    tracing::debug!("query_from_db: 2");

    let two: i32 = row.try_get(0).map_err(internal_error)?;
    tracing::debug!("query_from_db: 3");
    tracing::debug!("calc_result {:?}", two);

    Ok(two.to_string())
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
