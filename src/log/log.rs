use chrono::Local;
use std::fs::File;
use std::io::Write;

use crate::log::Log;

pub struct FileLog {
    file: File,
}

pub fn new_file_log(_file: &str) -> FileLog {
    let _file = File::create(_file).unwrap_or_else(|error| {
            print!("Не получилось открыть лог контроллера на запись: {:?}", error);
            File::create("/dev/null").unwrap()
        });
    FileLog { file: _file }
}

impl Log for FileLog {
    fn add_log(&mut self, msg: &str) {
        let time_stamp = Local::now().format("[%Y-%m-%d %H:%M:%S]");
        let formated_str = time_stamp.to_string() + "\t" + msg + "\n";

        self.file
            .write(formated_str.as_bytes())
            .expect("Ошибка записи в Лог");
    }
}
