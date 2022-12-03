use std::fs::File;
use std::io::{BufRead, BufReader};

use tokio::fs::read;

pub(crate) async fn read_file_async(filename: &str) -> anyhow::Result<Vec<u8>> {
    let mut local = filename;
    if local.starts_with('@') {
        local = &filename[1..]
    }
    Ok(read(local).await?)
}

pub(crate) fn read_file_lines_sync(filename: &str) -> anyhow::Result<Vec<String>> {
    let mut local = filename;
    if local.starts_with('@') {
        local = &filename[1..]
    }
    let fd = File::open(local)?;
    let mut reader = BufReader::new(fd);
    let mut result = Vec::new();
    loop {
        let mut s = String::new();
        if reader.read_line(&mut s)? == 0usize {
            break;
        }
        result.push(s.trim().to_string());
    }
    Ok(result)
}
