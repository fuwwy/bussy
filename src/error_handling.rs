use crate::guild_shell::LogData;

pub(crate) trait Loggable {
    fn slog(&mut self, content: String);
    fn log(&mut self, content: &str) {
        self.slog(content.to_string());
    }
}


pub trait BetterHandle<T> {
    fn dexpect(self, msg: &str, log: &mut LogData) -> T;
    fn dexpect_or_default(self, msg: &str, log: &mut LogData, default: T) -> T;
}

impl<T> BetterHandle<T> for Option<T> {
    fn dexpect(self, msg: &str, log: &mut LogData) -> T {
        match self {
            Some(val) => val,
            None => {
                log.log(&msg);
                panic!("Expect failed: {}", &msg)
            }
        }
    }

    fn dexpect_or_default(self, msg: &str, log: &mut LogData, default: T) -> T {
        match self {
            Some(val) => val,
            None => {
                log.log(&msg);
                println!("{}", msg);
                default
            }
        }
    }
}

impl<T, E> BetterHandle<T> for Result<T, E>
    where E: std::fmt::Display
{
    fn dexpect(self, msg: &str, log: &mut LogData) -> T {
        match self {
            Ok(val) => val,
            Err(e) => {
                log.log(msg);
                panic!("Expect failed because of {}: {}", e, &msg)
            }
        }
    }

    fn dexpect_or_default(self, msg: &str, log: &mut LogData, default: T) -> T {
        match self {
            Ok(val) => val,
            Err(e) => {
                log.slog(format!("{}: {}", &msg, e));
                println!("{}: {}", &msg, e);
                default
            }
        }
    }
}

