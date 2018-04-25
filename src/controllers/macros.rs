macro_rules! call_ctrl {
    ($ctrl_fn: expr) => (
        $ctrl_fn().and_then(|result| match result {
            Ok(result) => Ok(HttpResponse::Ok().json(result)),
            Err(err) => Err(Error::from(err)),
        })
        .responder()
    )
}
