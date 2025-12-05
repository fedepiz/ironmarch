#[macro_export]
macro_rules! get_or_return {
    ($e:expr) => {
        match $e {
            Some(x) => x,
            _ => {
                return;
            }
        }
    };
}

#[macro_export]
macro_rules! get_or_continue {
    ($e:expr, $msg:tt) => {
        match $e {
            Some(x) => x,
            _ => {
                println!($msg);
                continue;
            }
        }
    };
}
