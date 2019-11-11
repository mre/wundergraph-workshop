use diesel::pg::PgConnection;
use juniper::RootNode;
use wundergraph::prelude::*;
use wundergraph::scalar::WundergraphScalarValue;
use crate::model::users::User;

pub type Schema =
    RootNode<'static, Query<PgConnection>, Mutation<PgConnection>, WundergraphScalarValue>;

query_object! {
    /// Global query object for the schema
    Query {
        /// Access to User data 
        User,
    }
}

mutation_object!(Mutation {});

pub fn create_schema() -> Schema {
    Schema::new(Query::default(), Mutation::default())
}
