use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};

use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand, ValueEnum};
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
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Start the local disclosure PDF viewer")]
    Serve(ServeArgs),
    #[command(
        name = "fetch-segments",
        about = "Fetch segment data from JPX and save locally"
    )]
    FetchSegments,
}

#[derive(Debug, clap::Args)]
pub struct ServeArgs {
    #[arg(long, help = "Date to fetch (yyyymmdd)")]
    pub date: Option<String>,
    #[arg(long, default_value_t = DEFAULT_PORT, value_parser = parse_port)]
    pub port: u16,
    #[arg(long = "ticker-csv", help = "Path to ticker CSV file")]
    pub ticker_csv: Option<PathBuf>,
    #[arg(
        short = 't',
        long = "tickers",
        value_name = "CODE",
        num_args = 1..,
        help = "Ticker codes to filter"
    )]
    pub tickers: Vec<String>,
    #[arg(
        long = "segment",
        value_name = "NAME",
        num_args = 1..,
        help = "Market segments to filter (e.g. プライム)"
    )]
    pub segment: Vec<String>,
    #[arg(long, value_enum, default_value_t = Period::Day, help = "Period to fetch (day/week/month/year)")]
    pub period: Period,
    #[arg(
        long,
        conflicts_with_all = ["ticker_csv", "tickers", "segment"],
        help = "Show all disclosures, including ETF/ETN (conflicts with filters)"
    )]
    pub all: bool,
    #[arg(
        long = "kill-existing",
        help = "Kill an existing process listening on the selected port before starting"
    )]
    pub kill_existing: bool,
}

fn parse_port(value: &str) -> std::result::Result<u16, String> {
    let port = value
        .parse::<u16>()
        .map_err(|_| format!("invalid port: {value}"))?;
    if port == 0 {
        Err("port must be between 1 and 65535".to_string())
    } else {
        Ok(port)
    }
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
    if !open_browser_with_platform_default(url) {
        eprintln!("Could not open browser automatically. Open {url}");
    }
}

#[cfg(target_os = "windows")]
fn open_browser_with_platform_default(url: &str) -> bool {
    command_spawned("explorer.exe", &[url])
        || command_spawned("rundll32.exe", &["url.dll,FileProtocolHandler", url])
}

#[cfg(target_os = "macos")]
fn open_browser_with_platform_default(url: &str) -> bool {
    command_spawned("open", &[url])
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_browser_with_platform_default(url: &str) -> bool {
    command_spawned("xdg-open", &[url])
}

#[cfg(not(any(unix, target_os = "windows")))]
fn open_browser_with_platform_default(_url: &str) -> bool {
    false
}

fn command_spawned(program: &str, args: &[&str]) -> bool {
    ProcessCommand::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

#[cfg(target_os = "windows")]
fn find_listeners(port: u16) -> Vec<u32> {
    let output = match ProcessCommand::new("netstat").args(["-ano"]).output() {
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
    let output = match ProcessCommand::new("lsof")
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
    let _ = ProcessCommand::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .status();
}

#[cfg(not(target_os = "windows"))]
fn terminate_process(pid: u32) {
    let pid_arg = pid.to_string();
    let _ = ProcessCommand::new("kill")
        .args(["-TERM", &pid_arg])
        .status();
    std::thread::sleep(std::time::Duration::from_millis(500));
    if ProcessCommand::new("kill")
        .args(["-0", &pid_arg])
        .status()
        .is_ok_and(|status| status.success())
    {
        let _ = ProcessCommand::new("kill")
            .args(["-KILL", &pid_arg])
            .status();
    }
}

#[cfg(test)]
mod tests {
    use super::{Args, Period, load_tickers_from_csv};
    use clap::Parser;
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

    #[test]
    fn rejects_empty_filter_values() {
        assert!(Args::try_parse_from(["tdnet-viewer", "serve", "--tickers"]).is_err());
        assert!(Args::try_parse_from(["tdnet-viewer", "serve", "--segment"]).is_err());
    }

    #[test]
    fn rejects_conflicting_all_and_filters() {
        assert!(
            Args::try_parse_from(["tdnet-viewer", "serve", "--all", "--tickers", "7203"]).is_err()
        );
        assert!(
            Args::try_parse_from(["tdnet-viewer", "serve", "--all", "--segment", "プライム"])
                .is_err()
        );
    }

    #[test]
    fn rejects_port_zero() {
        assert!(Args::try_parse_from(["tdnet-viewer", "serve", "--port", "0"]).is_err());
    }
}
