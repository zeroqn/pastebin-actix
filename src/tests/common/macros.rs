macro_rules! sync_send {
    ($db_addr: expr, $msg: expr) => (
        $db_addr.send($msg).wait().unwrap().unwrap()
        )
}

macro_rules! parse_json {
    ($srv: expr, $res: expr, $type: ty) => {{
        let body = $srv.execute($res.body()).unwrap();
        serde_json::from_slice::<$type>(&body).unwrap()
    }}
}

macro_rules! assert_res_err {
    ($srv: expr, $req: expr, $code: expr, $assertion: expr) => {{
        let res = $srv.execute($req.send()).unwrap();
        assert!(res.status().is_client_error());

        let err_res = parse_json!($srv, res, ResponseError);
        assert_eq!(err_res.code, $code);
        $assertion(err_res);
    }}
}

macro_rules! assert_res_err_msg {
    ($srv: expr, $req: expr, $code: expr, $msg: expr) => {{
        assert_res_err!($srv, $req, $code, |res_err: ResponseError| {
            assert_eq!(res_err.msg, $msg);
        });
    }}
}

macro_rules! assert_res {
    ($srv: expr, $req: expr, $res_ty: ty, $assertion: expr) => {{
        let res = $srv.execute($req.send()).unwrap();
        assert!(res.status().is_success());

        $assertion(parse_json!($srv, res, $res_ty));
    }}
}
