use std::env;

use axum::{routing::get, Router};
use diesel::{dsl::insert_into, Insertable};
use diesel_async::{
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};
use dotenvy::dotenv;
use tokio::sync::OnceCell;

use axum_diesel_async_example::schema;

async fn build_connection_pool() -> Pool<AsyncPgConnection> {
    dotenv().ok();
    let connection_url = env::var("DATABASE_URL").unwrap();
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(connection_url);
    Pool::builder().build(manager).await.unwrap()
}

async fn get_connection_pool() -> &'static Pool<AsyncPgConnection> {
    static POOL: OnceCell<Pool<AsyncPgConnection>> = OnceCell::const_new();
    POOL.get_or_init(build_connection_pool).await
}

async fn add_book() -> String {
    #[derive(Insertable)]
    #[diesel(table_name = schema::book)]
    struct NewBook {
        title: String,
    }

    let conn_pool = get_connection_pool();
    let conn = &mut conn_pool.await.get().await.unwrap();

    insert_into(schema::book::table)
        .values(&NewBook {
            title: "book".to_owned(),
        })
        .execute(conn)
        .await
        .unwrap();

    "added book".to_owned()
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(add_book));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
