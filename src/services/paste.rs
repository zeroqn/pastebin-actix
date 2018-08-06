use std::time::SystemTime;

use actix::prelude::*;
use diesel::{self, prelude::*};

use crate::common::error::ServerError;
use crate::models::{
    executor::DatabaseExecutor as DbExecutor,
    paste::{NewPaste, Paste},
};

pub struct CreatePasteMsg {
    pub title: String,
    pub body: String,
    pub created_at: SystemTime,
}

impl Message for CreatePasteMsg {
    type Result = Result<Paste, ServerError>;
}

impl Handler<CreatePasteMsg> for DbExecutor {
    type Result = Result<Paste, ServerError>;

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
            .get_result(&self.0.get().map_err(ServerError::R2d2)?)
            .map_err(ServerError::Database)
    }
}

pub struct UpdatePasteMsg {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub modified_at: SystemTime,
}

impl Message for UpdatePasteMsg {
    type Result = Result<Paste, ServerError>;
}

impl Handler<UpdatePasteMsg> for DbExecutor {
    type Result = Result<Paste, ServerError>;

    fn handle(&mut self, msg: UpdatePasteMsg, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::pastes::dsl::*;

        diesel::update(pastes.find(msg.id))
            .set((
                title.eq(msg.title),
                body.eq(msg.body),
                modified_at.eq(msg.modified_at),
            )).get_result(&self.0.get().map_err(ServerError::R2d2)?)
            .map_err(ServerError::Database)
    }
}

pub struct GetPasteByIdMsg {
    pub id: i64,
}

impl Message for GetPasteByIdMsg {
    type Result = Result<Paste, ServerError>;
}

impl Handler<GetPasteByIdMsg> for DbExecutor {
    type Result = Result<Paste, ServerError>;

    fn handle(&mut self, msg: GetPasteByIdMsg, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::pastes::dsl::*;

        pastes
            .find(msg.id)
            .get_result(&self.0.get().map_err(ServerError::R2d2)?)
            .map_err(ServerError::Database)
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
    ($query:expr, $column:expr, $cmp:expr, $cond:expr) => {
        match $cmp {
            CmpOp::GT => $query.filter($column.gt($cond)),
            CmpOp::EQ => $query.filter($column.eq($cond)),
            CmpOp::LT => $query.filter($column.lt($cond)),
            CmpOp::GE => $query.filter($column.ge($cond)),
            CmpOp::LE => $query.filter($column.le($cond)),
        }
    };
}

macro_rules! order {
    ($query:expr, $column:expr, $order:expr) => {
        match $order {
            Order::Ascend => $query.order($column.asc()),
            Order::Decrease => $query.order($column.desc()),
        }
    };
}

macro_rules! orderby {
    ($query:expr, $column:expr, $order:expr) => {
        match $column {
            Item::Title => order!($query, title, $order),
            Item::Body => order!($query, body, $order),
            Item::CreatedAt => order!($query, created_at, $order),
            Item::ModifiedAt => order!($query, modified_at, $order),
        }
    };
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
    type Result = Result<Vec<Paste>, ServerError>;
}

impl Handler<GetPasteListMsg> for DbExecutor {
    type Result = Result<Vec<Paste>, ServerError>;

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

        query
            .load::<Paste>(&self.0.get().map_err(ServerError::R2d2)?)
            .map_err(ServerError::Database)
    }
}

pub struct DelPasteByIdMsg {
    pub id: i64,
}

impl Message for DelPasteByIdMsg {
    type Result = Result<usize, ServerError>;
}

impl Handler<DelPasteByIdMsg> for DbExecutor {
    type Result = Result<usize, ServerError>;

    fn handle(&mut self, msg: DelPasteByIdMsg, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::pastes::dsl::*;

        diesel::delete(pastes)
            .filter(id.eq(msg.id))
            .execute(&self.0.get().map_err(ServerError::R2d2)?)
            .map_err(ServerError::Database)
    }
}
