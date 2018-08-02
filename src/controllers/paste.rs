use actix_web::{AsyncResponder, HttpMessage, HttpRequest, HttpResponse, Query};
use futures::future::{self, Future};

use crate::common::{
    constant,
    error::{ServerError, UserError},
};
use crate::controllers::FutureJsonResponse;
use crate::services::paste as paste_srv;
use crate::State;

pub fn get_paste_by_id(req: &HttpRequest<State>) -> FutureJsonResponse {
    let db_chan = req.state().db_chan.clone();

    call_ctrl!(|| future::ok(req.clone())
        .and_then(|req| req.match_info()["id"].parse::<i64>())
        .from_err()
        .and_then(move |id| db_chan
            .send(paste_srv::GetPasteByIdMsg { id })
            .map_err(ServerError::MailBox)
            .from_err()))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetPasteListConds {
    title_pat: Option<String>,
    body_pat: Option<String>,
    cmp_created_at: Option<String>,
    cmp_modified_at: Option<String>,
    orderby_list: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

pub fn get_paste_list(
    (req, conds): (HttpRequest<State>, Query<GetPasteListConds>),
) -> FutureJsonResponse {
    let db_chan = req.state().db_chan.clone();
    let created_at = conds
        .cmp_created_at
        .to_owned()
        .map_or(Ok(None), |cmp_created_at| {
            parse_time_cond(&cmp_created_at).map(Option::from)
        });
    let modified_at = conds
        .cmp_modified_at
        .to_owned()
        .map_or(Ok(None), |cmp_modified_at| {
            parse_time_cond(&cmp_modified_at).map(Option::from)
        });
    let orderby_list = conds
        .orderby_list
        .to_owned()
        .map_or(Ok(None), |orderby_list| {
            parse_orderby(&orderby_list).map(Option::from)
        });
    let msg = paste_srv::GetPasteListMsg {
        title_pat: conds.title_pat.to_owned(),
        body_pat: conds.body_pat.to_owned(),
        limit: conds.limit,
        offset: conds.offset,
        ..Default::default()
    };

    call_ctrl!(|| future::ok(msg)
        .and_then(move |mut msg| created_at.map(|created_at| {
            msg.created_at = created_at;
            msg
        })).from_err()
        .and_then(move |mut msg| modified_at.map(|modified_at| {
            msg.modified_at = modified_at;
            msg
        })).from_err()
        .and_then(move |mut msg| orderby_list.map(|orderby_list| {
            msg.orderby_list = orderby_list;
            msg
        })).and_then(move |msg| db_chan.send(msg).map_err(ServerError::MailBox).from_err()))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NewPaste {
    pub title: String,
    pub body: String,
}

pub fn create_paste(req: &HttpRequest<State>) -> FutureJsonResponse {
    use std::time::SystemTime;

    let db_chan = req.state().db_chan.clone();

    // this requires correct content type
    call_ctrl!(|| req
        .json()
        .from_err()
        .and_then(move |new_paste: NewPaste| db_chan
            .send(paste_srv::CreatePasteMsg {
                title: new_paste.title,
                body: new_paste.body,
                created_at: SystemTime::now(),
            }).map_err(ServerError::MailBox)
            .from_err()))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdatePaste {
    pub id: i64,
    pub title: String,
    pub body: String,
}

pub fn update_paste_by_id(req: &HttpRequest<State>) -> FutureJsonResponse {
    use std::time::SystemTime;

    let db_chan = req.state().db_chan.clone();

    call_ctrl!(|| req
        .json()
        .from_err()
        .and_then(move |updated_paste: UpdatePaste| db_chan
            .send(paste_srv::UpdatePasteMsg {
                id: updated_paste.id,
                title: updated_paste.title,
                body: updated_paste.body,
                modified_at: SystemTime::now(),
            }).map_err(ServerError::MailBox)
            .from_err()))
}

pub fn del_paste_by_id(req: &HttpRequest<State>) -> FutureJsonResponse {
    let db_chan = req.state().db_chan.clone();

    call_ctrl!(|| future::ok(req.clone())
        .and_then(|req| req.match_info()["id"].parse::<i64>())
        .from_err()
        .and_then(move |id| db_chan
            .send(paste_srv::DelPasteByIdMsg { id })
            .map_err(ServerError::MailBox)
            .from_err()).map(|res| res.map(|_| "ok")))
}

// format: "GT/EQ/LT/GE/LE,seconds_since_UNIX_EPOCH"
fn parse_time_cond(cond_str: &str) -> Result<paste_srv::TimeCondition, UserError> {
    use self::paste_srv::{CmpOp, TimeCondition};
    use std::time::{Duration, UNIX_EPOCH};

    let default_err = Err(UserError::PayloadError(
        constant::ERR_MSG_PAYLOAD_PARSE_TIME_COND_FAIL.to_owned(),
    ));
    let op_secs: Vec<&str> = cond_str.split(',').collect();
    if op_secs.len() != 2 {
        return default_err;
    }

    let op = match op_secs[0] {
        "GT" => Ok(CmpOp::GT),
        "EQ" => Ok(CmpOp::EQ),
        "LT" => Ok(CmpOp::LT),
        "GE" => Ok(CmpOp::GE),
        "LE" => Ok(CmpOp::LE),
        _ => Err(()),
    };
    let secs = op_secs[1].parse::<u64>();

    if let (Ok(op), Ok(secs)) = (op, secs) {
        Ok(TimeCondition {
            op,
            time: UNIX_EPOCH + Duration::from_secs(secs),
        })
    } else {
        default_err
    }
}

// format: "Title/Body/CreatedAt/ModifiedAt:asc/decs"
fn parse_orderby(orderby_str: &str) -> Result<Vec<paste_srv::Orderby>, UserError> {
    use self::paste_srv::{Item, Order, Orderby};

    let default_err = Err(UserError::PayloadError(
        constant::ERR_MSG_PAYLOAD_PARSE_ORDERBY_FAIL.to_owned(),
    ));
    let comps: Vec<&str> = orderby_str.split(',').collect();
    if comps.is_empty() {
        return default_err;
    }
    let mut orderby_list: Vec<Orderby> = vec![];

    for comp in comps {
        let item_order: Vec<&str> = comp.split(':').collect();
        if item_order.len() != 2 {
            return default_err;
        }

        let item = match item_order[0] {
            "Title" => Ok(Item::Title),
            "Body" => Ok(Item::Body),
            "CreatedAt" => Ok(Item::CreatedAt),
            "ModifiedAt" => Ok(Item::ModifiedAt),
            _ => Err(()),
        };
        let order = match item_order[1] {
            "asc" => Ok(Order::Ascend),
            "decs" => Ok(Order::Decrease),
            _ => Err(()),
        };

        if let (Ok(item), Ok(order)) = (item, order) {
            orderby_list.push(Orderby { item, order })
        } else {
            return default_err;
        }
    }
    Ok(orderby_list)
}
