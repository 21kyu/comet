use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

const LOG_FILE_NAME: &str = "/var/log/comet-cni.log";

pub fn log(msg: &str) {
    let mut file = log_file();

    file.write_all(msg.as_bytes())
        .expect("Failed to write to log file");
    file.flush().expect("Failed to flush to log file");
}

fn log_file() -> File {
    return if let true = Path::new(LOG_FILE_NAME).exists() {
        OpenOptions::new()
            .append(true)
            .open(LOG_FILE_NAME)
            .expect("Failed to open log file")
    } else {
        File::create(LOG_FILE_NAME).expect("Failed to create log file")
    };
}
