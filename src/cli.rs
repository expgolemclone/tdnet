use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::config::DEFAULT_PORT;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Period {
    Day,
    Week,
    Month,
    Year,
}

impl Period {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
        }
    }

    pub fn target_days(self) -> usize {
        match self {
            Self::Day => 1,
            Self::Week => 5,
            Self::Month => 22,
            Self::Year => 250,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "day" => Some(Self::Day),
            "week" => Some(Self::Week),
            "month" => Some(Self::Month),
            "year" => Some(Self::Year),
            _ => None,
        }
    }

    pub fn valid_values() -> &'static [&'static str] {
        &["day", "week", "month", "year"]
    }
}

#[derive(Debug, Parser)]
#[command(about = "TDNet Disclosure PDF Viewer")]
pub struct Args {
    #[arg(long, help = "Date to fetch (yyyymmdd)")]
    pub date: Option<String>,
    #[arg(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
    #[arg(long = "ticker-csv", help = "Path to ticker CSV file")]
    pub ticker_csv: Option<PathBuf>,
    #[arg(short = 't', long = "tickers", num_args = 0.., help = "Ticker codes to filter")]
    pub tickers: Vec<String>,
    #[arg(long = "segment", num_args = 0.., help = "Market segments to filter (e.g. プライム)")]
    pub segment: Vec<String>,
    #[arg(
        long = "fetch-segments",
        help = "Fetch segment data from JPX and save locally"
    )]
    pub fetch_segments: bool,
    #[arg(long, value_enum, default_value_t = Period::Day, help = "Period to fetch (day/week/month/year)")]
    pub period: Period,
    #[arg(long, help = "Show all disclosures (no filter)")]
    pub all: bool,
}

pub fn today_yyyymmdd() -> String {
    Local::now().format("%Y%m%d").to_string()
}

pub fn load_tickers_from_csv(path: &Path) -> Result<HashSet<String>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read ticker CSV {}", path.display()))?;
    let mut tickers = HashSet::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(code) = line.split(',').next() {
            let code = code.trim();
            if !code.is_empty() {
                tickers.insert(code.to_string());
            }
        }
    }
    Ok(tickers)
}

pub fn kill_listeners(port: u16) {
    let my_pid = std::process::id();
    for pid in find_listeners(port) {
        if pid == my_pid {
            continue;
        }
        terminate_process(pid);
        println!("Killed old server process (PID {pid})");
    }
}

pub fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    let result = Command::new("cmd").args(["/C", "start", "", url]).status();
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(url).status();
    #[cfg(all(unix, not(target_os = "macos")))]
    let result = Command::new("xdg-open").arg(url).status();
    #[cfg(not(any(unix, target_os = "windows")))]
    let result = Ok(Default::default());

    if result.is_err() {
        eprintln!("Could not open browser automatically. Open {url}");
    }
}

#[cfg(target_os = "windows")]
fn find_listeners(port: u16) -> Vec<u32> {
    let output = match Command::new("netstat").args(["-ano"]).output() {
        Ok(output) => output,
        Err(_) => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<_> = line.split_whitespace().collect();
            if parts.len() < 5 || parts[3] != "LISTENING" {
                return None;
            }
            if !parts[1].contains(&format!(":{port}")) {
                return None;
            }
            parts[4].parse::<u32>().ok()
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn find_listeners(port: u16) -> Vec<u32> {
    let output = match Command::new("lsof")
        .args(["-i", &format!(":{port}"), "-t", "-sTCP:LISTEN"])
        .output()
    {
        Ok(output) => output,
        Err(_) => return Vec::new(),
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse::<u32>().ok())
        .collect()
}

#[cfg(target_os = "windows")]
fn terminate_process(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .status();
}

#[cfg(not(target_os = "windows"))]
fn terminate_process(pid: u32) {
    let pid_arg = pid.to_string();
    let _ = Command::new("kill").args(["-TERM", &pid_arg]).status();
    thread::sleep(Duration::from_millis(500));
    if Command::new("kill")
        .args(["-0", &pid_arg])
        .status()
        .is_ok_and(|status| status.success())
    {
        let _ = Command::new("kill").args(["-KILL", &pid_arg]).status();
    }
}

#[cfg(test)]
mod tests {
    use super::{Period, load_tickers_from_csv};
    use std::fs;

    #[test]
    fn parses_period_values() {
        assert_eq!(Period::parse("day"), Some(Period::Day));
        assert_eq!(Period::parse("week"), Some(Period::Week));
        assert_eq!(Period::parse("bad"), None);
    }

    #[test]
    fn loads_ticker_csv() {
        let path = std::env::temp_dir().join(format!("tdnet-tickers-{}.csv", std::process::id()));
        fs::write(&path, "# comment\n1234,foo\n\n5678\n").unwrap();
        let tickers = load_tickers_from_csv(&path).unwrap();
        let _ = fs::remove_file(&path);
        assert!(tickers.contains("1234"));
        assert!(tickers.contains("5678"));
        assert_eq!(tickers.len(), 2);
    }
}
