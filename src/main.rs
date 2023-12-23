use anyhow::Context;
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    let mut vtt = String::new();
    io::stdin().read_to_string(&mut vtt)?;

    let content = vtt
        .strip_prefix("WEBVTT\r\n\r\n")
        .context("this is not a valid WEBVTT file")?;

    let mut writer = csv::Writer::from_writer(io::stdout());
    writer.write_record(["id", "start", "end", "text"])?;

    for (i, line) in content.split("\r\n\r\n").enumerate() {
        if line.trim().is_empty() {
            break;
        }

        let (id, start, end, text) = extract_line_infos(line)
            .with_context(|| format!("an error occured on the line `{line:?}` around line {i}"))?;
        writer.write_record([id, &start.to_string(), &end.to_string(), text])?;
    }

    writer.flush()?;

    Ok(())
}

fn extract_line_infos(line: &str) -> anyhow::Result<(&str, u64, u64, &str)> {
    let (id, line) = line.split_once("\r\n").context("invalid line id format")?;
    let (start_end, text) = line
        .split_once("\r\n")
        .context("cannot split by \\r\\n")
        .context("invalid line start-end format")?;
    let (start, end) = start_end
        .split_once(" --> ")
        .context("cannot split by arrow (` --> `)")
        .context("invalid line start-end format")?;

    let start_milliseconds =
        convert_timer_in_milliseconds(start).context("invalid timer format")?;
    let end_milliseconds = convert_timer_in_milliseconds(end).context("invalid timer format")?;

    Ok((id, start_milliseconds, end_milliseconds, text))
}

fn convert_timer_in_milliseconds(timer: &str) -> Option<u64> {
    let (hours, timer) = timer.split_once(':')?;
    let (minutes, timer) = timer.split_once(':')?;
    let (seconds, milliseconds) = timer.split_once('.')?;

    let hours: u64 = hours.parse().ok()?;
    let minutes: u64 = minutes.parse().ok()?;
    let seconds: u64 = seconds.parse().ok()?;
    let milliseconds: u64 = milliseconds.parse().ok()?;

    Some(hours * 60 * 60 * 1000 + minutes * 60 * 1000 + seconds * 1000 + milliseconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convertions() {
        assert_eq!(
            convert_timer_in_milliseconds("01:09:33.252"),
            Some(60 * 60 * 1000 + 9 * 60 * 1000 + 33 * 1000 + 252)
        );
        assert_eq!(
            convert_timer_in_milliseconds("01:09:36.547"),
            Some(60 * 60 * 1000 + 9 * 60 * 1000 + 36 * 1000 + 547)
        );
    }

    #[test]
    fn extract_vtt_line_infos() {
        let line = "1\r\n00:00:20.672 --> 00:00:24.972\r\nEntre l’Australia et la South America,
dans l’Océan South Pacific…";
        println!("{:?}", line);

        let (id, start, end, text) = extract_line_infos(line).unwrap();
        assert_eq!(id, "1");
        assert_eq!(start, 20672);
        assert_eq!(end, 24972);
        assert_eq!(
            text,
            "Entre l’Australia et la South America,
dans l’Océan South Pacific…"
        );
    }
}
