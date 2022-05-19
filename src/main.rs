use anyhow::Context;
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    let mut vtt = String::new();
    io::stdin().read_to_string(&mut vtt)?;

    let content = vtt
        .strip_prefix("WEBVTT\r\n\r\n")
        .context("this is not a valid WEBVTT file")?;

    let mut writer = csv::Writer::from_writer(io::stdout());
    writer.write_record(&["id", "start", "end", "text"])?;

    for content in content.split("\r\n\r\n") {
        if content.trim().is_empty() {
            break;
        }

        let (id, content) = content
            .split_once("\r\n")
            .context("invalid line id format")?;
        let (start_end, text) = content
            .split_once("\r\n")
            .context("invalid line start-end format")?;
        let (start, end) = start_end
            .split_once(" --> ")
            .context("invalid line start-end format")?;
        writer.write_record(&[id, start, end, text])?;
    }

    writer.flush()?;

    Ok(())
}
