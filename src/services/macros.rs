#[cfg(test)]
macro_rules! sync_send {
    ($db_addr: expr, $msg: expr) => (
        $db_addr.send($msg).wait().unwrap().unwrap()
        )
}
