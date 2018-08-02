use std::time::SystemTime;

use diesel;
use diesel::prelude::*;
use diesel::result::Error as DieselError;

use actix::prelude::*;

use crate::db::executor::DbExecutor;
use crate::models::paste::{NewPaste, Paste};

pub struct CreatePasteMsg {
    pub title: String,
    pub body: String,
    pub created_at: SystemTime,
}

impl Message for CreatePasteMsg {
    type Result = Result<Paste, DieselError>;
}

impl Handler<CreatePasteMsg> for DbExecutor {
    type Result = Result<Paste, DieselError>;

    fn handle(&mut self, msg: CreatePasteMsg, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::pastes::dsl::*;

        let new_paste = NewPaste {
            title: &msg.title,
            body: &msg.body,
            created_at: &msg.created_at,
            modified_at: &msg.created_at,
        };

        diesel::insert_into(pastes)
            .values(&new_paste)
            .get_result(self.conn())
    }
}

pub struct UpdatePasteMsg {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub modified_at: SystemTime,
}

impl Message for UpdatePasteMsg {
    type Result = Result<Paste, DieselError>;
}

impl Handler<UpdatePasteMsg> for DbExecutor {
    type Result = Result<Paste, DieselError>;

    fn handle(&mut self, msg: UpdatePasteMsg, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::pastes::dsl::*;

        diesel::update(pastes.find(msg.id))
            .set((
                title.eq(msg.title),
                body.eq(msg.body),
                modified_at.eq(msg.modified_at),
            ))
            .get_result(self.conn())
    }
}

pub struct GetPasteByIdMsg {
    pub id: i64,
}

impl Message for GetPasteByIdMsg {
    type Result = Result<Paste, DieselError>;
}

impl Handler<GetPasteByIdMsg> for DbExecutor {
    type Result = Result<Paste, DieselError>;

    fn handle(&mut self, msg: GetPasteByIdMsg, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::pastes::dsl::*;

        pastes.find(msg.id).get_result(self.conn())
    }
}

#[derive(Debug)]
pub enum Item {
    Title,
    Body,
    CreatedAt,
    ModifiedAt,
}

#[derive(Debug)]
pub enum Order {
    Ascend,
    Decrease,
}

#[derive(Debug)]
pub enum CmpOp {
    GT,
    EQ,
    LT,
    GE,
    LE,
}

#[derive(Debug)]
pub struct Orderby {
    pub item: Item,
    pub order: Order,
}

#[derive(Debug)]
pub struct TimeCondition {
    pub op: CmpOp,
    pub time: SystemTime,
}

macro_rules! cmp {
    ($query: expr, $column: expr, $cmp: expr, $cond: expr) => (
        match $cmp {
            CmpOp::GT => $query.filter($column.gt($cond)),
            CmpOp::EQ => $query.filter($column.eq($cond)),
            CmpOp::LT => $query.filter($column.lt($cond)),
            CmpOp::GE => $query.filter($column.ge($cond)),
            CmpOp::LE => $query.filter($column.le($cond))
        }
    )
}

macro_rules! order {
    ($query: expr, $column: expr, $order: expr) => (
        match $order {
            Order::Ascend => $query.order($column.asc()),
            Order::Decrease => $query.order($column.desc())
        }
    )
}

macro_rules! orderby {
    ($query: expr, $column: expr, $order: expr) => (
        match $column {
            Item::Title => order!($query, title, $order),
            Item::Body => order!($query, body, $order),
            Item::CreatedAt => order!($query, created_at, $order),
            Item::ModifiedAt => order!($query, modified_at, $order)
        }
    )
}

pub struct GetPasteListMsg {
    pub title_pat: Option<String>,
    pub body_pat: Option<String>,
    pub created_at: Option<TimeCondition>,
    pub modified_at: Option<TimeCondition>,
    pub orderby_list: Option<Vec<Orderby>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for GetPasteListMsg {
    fn default() -> Self {
        GetPasteListMsg {
            title_pat: None,
            body_pat: None,
            created_at: None,
            modified_at: None,
            orderby_list: None,
            limit: Some(20),
            offset: Some(0),
        }
    }
}

impl Message for GetPasteListMsg {
    type Result = Result<Vec<Paste>, DieselError>;
}

impl Handler<GetPasteListMsg> for DbExecutor {
    type Result = Result<Vec<Paste>, DieselError>;

    fn handle(&mut self, msg: GetPasteListMsg, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::pastes::dsl::*;

        let mut query = pastes.into_boxed();

        if let Some(title_pat) = msg.title_pat {
            query = query.filter(title.ilike(title_pat.to_owned() + "%"));
        }

        if let Some(body_pat) = msg.body_pat {
            query = query.filter(body.ilike(body_pat.to_owned() + "%"));
        }

        if let Some(cond) = msg.created_at {
            query = cmp!(query, created_at, cond.op, cond.time);
        }

        if let Some(cond) = msg.modified_at {
            query = cmp!(query, modified_at, cond.op, cond.time);
        }

        if let Some(orderby_list) = msg.orderby_list {
            for orderby in orderby_list {
                query = orderby!(query, orderby.item, orderby.order);
            }
        }

        if let Some(limit) = msg.limit {
            query = query.limit(limit);
        }

        if let Some(offset) = msg.offset {
            query = query.offset(offset);
        }

        query.load::<Paste>(self.conn())
    }
}

pub struct DelPasteByIdMsg {
    pub id: i64,
}

impl Message for DelPasteByIdMsg {
    type Result = Result<usize, DieselError>;
}

impl Handler<DelPasteByIdMsg> for DbExecutor {
    type Result = Result<usize, DieselError>;

    fn handle(&mut self, msg: DelPasteByIdMsg, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::pastes::dsl::*;

        diesel::delete(pastes)
            .filter(id.eq(msg.id))
            .execute(self.conn())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use futures::future::Future;

    use crate::TEST_DB_CHAN;

    #[test]
    fn test_create_paste() {
        let db_chan = TEST_DB_CHAN.clone();
        let test_title = "test unamed";
        let test_body = "test pastebin";

        let added_paste = sync_send!(
            db_chan,
            CreatePasteMsg {
                title: test_title.to_owned(),
                body: test_body.to_owned(),
                created_at: SystemTime::now(),
            }
        );

        assert_eq!(added_paste.title, test_title);
        assert_eq!(added_paste.body, test_body);
    }

    #[test]
    fn test_update_paste() {
        let db_chan = TEST_DB_CHAN.clone();
        let test_title = "test unamed";
        let test_body = "test pastebin";
        let updated_body = "test updated pastebin";

        let added_paste = sync_send!(
            db_chan,
            CreatePasteMsg {
                title: test_title.to_owned(),
                body: test_body.to_owned(),
                created_at: SystemTime::now(),
            }
        );

        assert_eq!(added_paste.title, test_title);
        assert_eq!(added_paste.body, test_body);

        let updated_paste = sync_send!(
            db_chan,
            UpdatePasteMsg {
                id: added_paste.id,
                title: added_paste.title.to_owned(),
                body: updated_body.to_owned(),
                modified_at: SystemTime::now(),
            }
        );

        assert_eq!(updated_paste.title, test_title);
        assert_eq!(updated_paste.body, updated_body);
    }

    #[test]
    fn test_get_paste_by_id() {
        let db_chan = TEST_DB_CHAN.clone();
        let test_title = "test unamed";
        let test_body = "test pastebin";

        let added_paste = sync_send!(
            db_chan,
            CreatePasteMsg {
                title: test_title.to_owned(),
                body: test_body.to_owned(),
                created_at: SystemTime::now(),
            }
        );

        assert_eq!(added_paste.title, test_title);
        assert_eq!(added_paste.body, test_body);

        let paste = sync_send!(db_chan, GetPasteByIdMsg { id: added_paste.id });
        assert_eq!(paste.title, test_title);
        assert_eq!(paste.body, test_body);
    }

    #[test]
    fn test_get_paste_list() {
        use crate::tests::testdata;

        let db_chan = TEST_DB_CHAN.clone();
        let test_paste_list = testdata::recreate();

        // By default, fetch 20 pastes
        let mut fetched_paste_list = sync_send!(
            db_chan,
            GetPasteListMsg {
                ..Default::default()
            }
        );

        assert_eq!(fetched_paste_list.len(), test_paste_list.len());
        for paste in test_paste_list {
            assert!(fetched_paste_list.contains(&paste));
        }

        // Fetch pastes, their titles contain "test title" and bodies
        // contains "test body 1".
        fetched_paste_list = sync_send!(
            db_chan,
            GetPasteListMsg {
                title_pat: Some("test title".to_owned()),
                body_pat: Some("test body 1".to_owned()),
                ..Default::default()
            }
        );
        assert_eq!(fetched_paste_list.len(), 1);
        assert_eq!(fetched_paste_list.pop().unwrap().title, "test title 1");

        // Fetch pates, their titles contain "test title", their creation
        // time should before 10 seconds from now and order by their titles.
        fetched_paste_list = sync_send!(
            db_chan,
            GetPasteListMsg {
                title_pat: Some("test title".to_owned()),
                created_at: Some(TimeCondition {
                    op: CmpOp::LE,
                    time: SystemTime::now() + Duration::new(10, 0),
                }),
                orderby_list: Some(vec![
                    Orderby {
                        item: Item::Title,
                        order: Order::Ascend,
                    },
                ]),
                limit: Some(5),
                ..Default::default()
            }
        );

        assert_eq!(fetched_paste_list.len(), 5);
        for (idx, paste) in fetched_paste_list.iter().enumerate() {
            assert_eq!(
                paste.title,
                "test title ".to_owned() + &(idx + 1).to_string()
            );
        }
    }

    #[test]
    fn test_delte_paste_by_id() {
        let db_chan = TEST_DB_CHAN.clone();
        let test_title = "test unamed";
        let test_body = "test pastebin";

        let added_paste = sync_send!(
            db_chan,
            CreatePasteMsg {
                title: test_title.to_owned(),
                body: test_body.to_owned(),
                created_at: SystemTime::now(),
            }
        );

        assert_eq!(added_paste.title, test_title);
        assert_eq!(added_paste.body, test_body);

        let count = sync_send!(db_chan, DelPasteByIdMsg { id: added_paste.id });
        assert_eq!(1, count);
    }
}
