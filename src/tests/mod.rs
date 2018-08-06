use std::sync::{Arc, Mutex, MutexGuard};

use actix::prelude::*;
// prelude is required for PgConnection::establish()
use diesel::{
    pg::PgConnection,
    prelude::*,
    r2d2::{ConnectionManager, CustomizeConnection, Error as R2d2Error, Pool},
};

use crate::models::{executor::DatabaseExecutor as DBExecutor, paste::Paste};
use crate::ENV;

#[macro_use]
pub mod macros;
pub mod paste;

lazy_static! {
    static ref TEST_SUIT: TestSuit = TestSuit::new(&ENV.database_url);
}

#[derive(Debug)]
pub struct TestTxConnCustomizer;

impl CustomizeConnection<PgConnection, R2d2Error> for TestTxConnCustomizer {
    fn on_acquire(&self, conn: &mut PgConnection) -> Result<(), R2d2Error> {
        conn.begin_test_transaction().map_err(R2d2Error::QueryError)
    }
}

pub struct ResetPool();

impl Message for ResetPool {
    type Result = Result<(), ()>;
}

impl Handler<ResetPool> for DBExecutor {
    type Result = Result<(), ()>;

    fn handle(&mut self, _msg: ResetPool, _: &mut Self::Context) -> Self::Result {
        // replace old pool with newly created one
        self.0 = TestSuit::create_pool(&TEST_SUIT.database_url);
        Ok(())
    }
}

struct TestSuit {
    database_url: String,
    data: Vec<Paste>,
    executor: Addr<DBExecutor>,
    locker: Arc<Mutex<()>>,
}

impl TestSuit {
    pub fn new(database_url: &'static str) -> Self {
        let data = Self::create_data(database_url);
        let pool = Self::create_pool(database_url);
        let executor = Self::create_executor(pool);

        TestSuit {
            database_url: database_url.to_owned(),
            data,
            executor,
            locker: Arc::new(Mutex::new(())),
        }
    }

    pub fn data(&self) -> &Vec<Paste> {
        &self.data
    }

    pub fn executor(&self) -> Addr<DBExecutor> {
        self.executor.clone()
    }

    pub fn begin_isolated_test(&self) -> MutexGuard<()> {
        use futures::future::Future;

        let guard = match self.locker.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        sync_send!(self.executor, ResetPool {});

        guard
    }

    pub fn create_pool(database_url: &'static str) -> Pool<ConnectionManager<PgConnection>> {
        let manager = ConnectionManager::<PgConnection>::new(database_url.to_owned());
        Pool::builder()
            .max_size(1)
            .connection_customizer(Box::new(TestTxConnCustomizer))
            .build(manager)
            .expect("cannot build database connection pool")
    }

    pub fn create_executor(pool: Pool<ConnectionManager<PgConnection>>) -> Addr<DBExecutor> {
        use std::sync::mpsc::channel;
        use std::thread;

        let (tx, rx) = channel();

        thread::spawn(move || {
            let sys = actix::System::new("pastebin-test");

            let addr = SyncArbiter::start(1, move || DBExecutor(pool.clone()));
            tx.send(addr).unwrap();

            sys.run();
        });

        rx.recv().unwrap()
    }

    pub fn create_data(database_url: &str) -> Vec<Paste> {
        use crate::models::{paste::NewPaste, schema::pastes::dsl::*};
        use std::time::SystemTime;

        let conn = PgConnection::establish(database_url).unwrap();

        let now = SystemTime::now();
        let paste_list = (1..10)
            .map(|n| {
                (
                    "test title ".to_owned() + &n.to_string(),
                    "test body ".to_owned() + &n.to_string(),
                )
            }).collect::<Vec<(_, _)>>();
        let new_paste_list = (0..9)
            .map(|i| {
                let paste = paste_list.get(i).unwrap();
                NewPaste {
                    title: &paste.0,
                    body: &paste.1,
                    created_at: &now,
                    modified_at: &now,
                }
            }).collect::<Vec<_>>();

        diesel::delete(pastes)
            .execute(&conn)
            .expect("fail to clear table");

        diesel::insert_into(pastes)
            .values(&new_paste_list)
            .get_results(&conn)
            .expect("fail to insert test data")
    }
}
