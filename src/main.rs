#[macro_use]
extern crate diesel;

extern crate juniper;

use actix_web::web::{Data, Json};
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use juniper::http::GraphQLRequest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use structopt::StructOpt;
use wundergraph::scalar::WundergraphScalarValue;

mod graphql_schema;
mod model;
mod pagination;
#[allow(unused_imports)]
mod schema;
#[macro_use]
mod diesel_ext;

use crate::graphql_schema::{create_schema, Schema};

#[derive(Debug, StructOpt)]
#[structopt(name = "rustfest")]
struct Opt {
    #[structopt(short = "u", long = "db-url")]
    database_url: String,
    #[structopt(short = "s", long = "socket", default_value = "127.0.0.1:8000")]
    socket: String,
}

#[derive(Clone)]
struct AppState {
    pool: Pool<ConnectionManager<PgConnection>>,
    schema: Arc<Schema>,
}

// #[derive(Debug)]
// pub struct MyContext<Conn>
// where
//     Conn: Connection + 'static,
// {
//     conn: PooledConnection<ConnectionManager<Conn>>,
// }

// impl<Conn> MyContext<Conn>
// where
//     Conn: Connection + 'static,
// {
//     pub fn new(conn: PooledConnection<ConnectionManager<Conn>>) -> Self {
//         Self { conn }
//     }
// }


// actix integration stuff
#[derive(Serialize, Deserialize, Debug)]
pub struct GraphQLData(GraphQLRequest<WundergraphScalarValue>);

fn graphql(
    Json(GraphQLData(data)): Json<GraphQLData>,
    st: Data<AppState>,
) -> Result<HttpResponse, failure::Error> {
    let ctx = st.get_ref().pool.get()?;

    let res = data.execute(&*st.get_ref().schema, &ctx);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&res)?))
}

fn main() {
    let opt = Opt::from_args();
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let manager = ConnectionManager::<PgConnection>::new(opt.database_url);
    let pool = Pool::builder().build(manager).expect("Failed to init pool");

    diesel_migrations::run_pending_migrations(&pool.get().expect("Failed to get db connection"))
        .expect("Failed to run migrations");

    // Create Juniper schema
    let schema = std::sync::Arc::new(create_schema());
    let data = AppState { pool, schema };

    let url = opt.socket;

    println!("Started http server: http://{}", url);

    HttpServer::new(move || {
        App::new()
            .configure(model::posts::config)
            .configure(model::users::config)
            .configure(model::comments::config)
            .data(data.clone())
            .route("/graphql", web::get().to(graphql))
            .route("/graphql", web::post().to(graphql))
            .wrap(middleware::Logger::default())
    })
    .bind(&url)
    .expect("Failed to start server")
    .run()
    .unwrap();
}
