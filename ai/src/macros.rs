#[macro_export]
macro_rules! unwrap_or_err {
    ($v:expr) => {
        match $v {
            Some(value) => value,
            None => return Err(()),
        }
    };
}
