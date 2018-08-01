// prelude is required for PgConnection::establish()
use diesel::pg::PgConnection;
use diesel::prelude::*;

pub fn establish_connection(database_url: &str) -> PgConnection {
    PgConnection::establish(database_url).expect(&format!("Error connection to {}", database_url))
}

pub mod executor {
    use super::PgConnection;
    use actix::prelude::*;

    pub struct DbExecutor(PgConnection);

    impl Actor for DbExecutor {
        type Context = SyncContext<Self>;
    }

    impl DbExecutor {
        pub fn new(conn: PgConnection) -> Self {
            DbExecutor(conn)
        }

        pub fn conn(&self) -> &PgConnection {
            &self.0
        }
    }
}
