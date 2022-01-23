pub mod log;

pub trait Log {
    fn add_log(&mut self, msg: &str);
}
