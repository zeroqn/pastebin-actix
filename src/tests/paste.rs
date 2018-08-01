use actix_web::{App, HttpMessage, http::Method};
use actix_web::test::TestServer;

use serde_json;

use State;
use TEST_DB_CHAN;

use apps::paste as paste_app;
use controllers::paste::{NewPaste, UpdatePaste};
use models::paste::Paste;
use common::{constant::*, error::ResponseError};
use tests::testdata;

fn create_app() -> App<State> {
    paste_app::create(State {
        db_chan: TEST_DB_CHAN.clone(),
    })
}

fn init_server() -> TestServer {
    TestServer::with_factory(create_app)
}

#[test]
fn test_get_paste_by_id() {
    let paste_list = testdata::recreate();
    let paste = paste_list.first().unwrap();

    let mut srv = init_server();

    let req = srv.client(Method::GET, &format!("/pastes/{}", paste.id))
        .finish()
        .unwrap();

    assert_res!(srv, req, Paste, |fetched_paste: Paste| {
        assert_eq!(fetched_paste.id, paste.id);
        assert_eq!(fetched_paste.title, paste.title);
        assert_eq!(fetched_paste.body, paste.body);
    });
}

#[test]
fn test_get_paste_by_bad_id() {
    let mut srv = init_server();

    let req = srv.client(Method::GET, &format!("/pastes/{}", "dddd"))
        .finish()
        .unwrap();

    assert_res_err_msg!(srv, req, 400, ERR_MSG_BAD_ID);
}

#[test]
fn test_get_paste_by_none_exist_id() {
    let mut srv = init_server();

    let req = srv.client(Method::GET, &format!("/pastes/{}", 99999999))
        .finish()
        .unwrap();

    assert_res_err_msg!(srv, req, 404, ERR_MSG_DATA_NOT_FOUND);
}

#[test]
fn test_get_paste_list() {
    testdata::recreate();
    let mut srv = init_server();

    let req = srv.client(
        Method::GET,
        &format!(
            "/pastes?title_pat{}&body_pat{}&limit={}&cmp_created_at={}&orderby_list={}",
            "test", "test body", 5, "GT%2C100000", "Title%3Aasc%2CBody%3Aasc"
        ),
    ).finish()
        .unwrap();

    assert_res!(srv, req, Vec<Paste>, |pastes: Vec<Paste>| {
        assert_eq!(pastes.len(), 5);
        for (idx, paste) in pastes.iter().enumerate() {
            assert!(paste.title.contains("test"));
            assert!(paste.body.contains("test body"));
            assert_eq!(
                paste.title,
                "test title ".to_string() + &(idx + 1).to_string()
            );
            assert_eq!(
                paste.body,
                "test body ".to_string() + &(idx + 1).to_string()
            );
        }
    });
}

#[test]
fn test_get_paste_list_with_bad_cmp_created_at() {
    let mut srv = init_server();

    let req = srv.client(
        Method::GET,
        &format!(
            "/pastes?title_pat{}&body_pat{}&limit={}&cmp_created_at={}",
            "test", "test body", 5, "DD%2C100000"
        ),
    ).finish()
        .unwrap();

    assert_res_err_msg!(srv, req, 400, ERR_MSG_PAYLOAD_PARSE_TIME_COND_FAIL);
}

#[test]
fn test_get_paste_list_with_bad_cmp_modified_at() {
    let mut srv = init_server();

    let req = srv.client(
        Method::GET,
        &format!(
            "/pastes?title_pat{}&body_pat{}&limit={}&cmp_modified_at={}",
            "test", "test body", 5, "DD%2C100000"
        ),
    ).finish()
        .unwrap();

    assert_res_err_msg!(srv, req, 400, ERR_MSG_PAYLOAD_PARSE_TIME_COND_FAIL);
}

#[test]
fn test_get_paste_list_with_bad_orderby_list() {
    let mut srv = init_server();

    let req = srv.client(
        Method::GET,
        &format!(
            "/pastes?title_pat{}&body_pat{}&limit={}&orderby_list={}",
            "test", "test body", 5, "BAD%3Aasc"
        ),
    ).finish()
        .unwrap();

    assert_res_err_msg!(srv, req, 400, ERR_MSG_PAYLOAD_PARSE_ORDERBY_FAIL);
}

#[test]
fn test_creat_paste() {
    let mut srv = init_server();

    let req = srv.client(Method::POST, "/pastes")
        .content_type(CONTENT_TYPE_JSON)
        .body(
            serde_json::to_vec(&NewPaste {
                title: "test new paste".to_string(),
                body: "my new paste".to_string(),
            }).unwrap(),
        )
        .unwrap();

    assert_res!(srv, req, Paste, |created_paste: Paste| {
        assert!(created_paste.id > 0);
        assert_eq!(created_paste.title, "test new paste");
        assert_eq!(created_paste.body, "my new paste");
    });
}

#[test]
fn test_create_paste_with_bad_payload() {
    let mut srv = init_server();

    let req = srv.client(Method::POST, "/pastes")
        .content_type(CONTENT_TYPE_JSON)
        .body("{\"bad\": \"bad payload\"}")
        .unwrap();

    assert_res_err!(srv, req, 400, |res: ResponseError| {
        assert!(res.msg.contains("Json deserialize error"));
    });
}

#[test]
fn test_update_paste() {
    let paste_list = testdata::recreate();
    let paste = paste_list.first().unwrap();

    let mut srv = init_server();

    let req = srv.client(Method::POST, &format!("/pastes/{}", paste.id))
        .content_type(CONTENT_TYPE_JSON)
        .json(UpdatePaste {
            id: paste.id,
            title: "test updated paste".to_string(),
            body: "test updated ddd body".to_string(),
        })
        .unwrap();

    assert_res!(srv, req, Paste, |updated_paste: Paste| {
        assert!(updated_paste.id == paste.id);
        assert_eq!(updated_paste.title, "test updated paste");
        assert_eq!(updated_paste.body, "test updated ddd body");
    });
}

#[test]
fn test_update_paste_with_bad_payload() {
    let paste_list = testdata::recreate();
    let paste = paste_list.first().unwrap();

    let mut srv = init_server();

    let req = srv.client(Method::POST, &format!("/pastes/{}", paste.id))
        .content_type(CONTENT_TYPE_JSON)
        .body("{\"id\": \"dddd\"}")
        .unwrap();

    assert_res_err!(srv, req, 400, |res: ResponseError| {
        assert!(res.msg.contains("Json deserialize error"));
    });
}

#[test]
fn test_del_paste_by_id() {
    let paste_list = testdata::recreate();
    let paste = paste_list.first().unwrap();

    let mut srv = init_server();

    let req = srv.client(Method::DELETE, &format!("/pastes/{}", paste.id))
        .finish()
        .unwrap();

    assert_res!(srv, req, String, |res: String| {
        assert_eq!(res, "ok");
    });
}

#[test]
fn test_del_paste_by_bad_id() {
    testdata::recreate();

    let mut srv = init_server();
    let req = srv.client(Method::DELETE, &format!("/pastes/{}", "dddd"))
        .finish()
        .unwrap();

    assert_res_err_msg!(srv, req, 400, ERR_MSG_BAD_ID);
}
