import argparse
import os
import re
import subprocess
import sys
import webbrowser
from datetime import date

import truststore

truststore.inject_into_ssl()

from config import DEFAULT_PORT, VALID_PERIODS
from segments import fetch_jpx_segments, load_segments, save_segments
from server import app, cache_set, date_range, set_exclude_tickers, set_ticker_filter
from scraper import fetch_disclosures


def _find_listeners_windows(port: int) -> set[int]:
    """Find PIDs listening on *port* using Windows netstat."""
    try:
        out = subprocess.check_output(
            ["netstat", "-ano"], stderr=subprocess.DEVNULL,
        ).decode(errors="ignore")
    except (OSError, subprocess.CalledProcessError):
        return set()
    pattern = re.compile(rf"LISTENING\s+(\d+)\s*$")
    port_str = f":{port}"
    pids: set[int] = set()
    for line in out.splitlines():
        if port_str not in line:
            continue
        m = pattern.search(line)
        if m:
            pids.add(int(m.group(1)))
    return pids


def _find_listeners_unix(port: int) -> set[int]:
    """Find PIDs listening on *port* using lsof (Linux/macOS)."""
    try:
        out = subprocess.check_output(
            ["lsof", "-i", f":{port}", "-t"], stderr=subprocess.DEVNULL,
        ).decode(errors="ignore")
    except (OSError, subprocess.CalledProcessError):
        return set()
    pids: set[int] = set()
    for line in out.splitlines():
        line = line.strip()
        if line.isdigit():
            pids.add(int(line))
    return pids


def kill_listeners(port: int) -> None:
    """Kill processes listening on the given port (except ourselves)."""
    my_pid = os.getpid()
    if sys.platform == "win32":
        pids = _find_listeners_windows(port)
    else:
        pids = _find_listeners_unix(port)
    pids.discard(my_pid)
    for pid in pids:
        try:
            os.kill(pid, 9)
            print(f"Killed old server process (PID {pid})")
        except OSError:
            pass


def load_tickers_from_csv(path: str) -> set[str]:
    """Load ticker codes from a CSV file (one per line, # comments)."""
    tickers: set[str] = set()
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            tickers.add(line.split(",")[0].strip())
    return tickers


def build_ticker_filter(args) -> set[str] | None:
    """Build a ticker filter set from CLI args. Returns None for no filter."""
    if args.all:
        return None

    ticker_set: set[str] = set()
    if args.ticker_csv:
        ticker_set |= load_tickers_from_csv(args.ticker_csv)
    if args.tickers:
        ticker_set.update(args.tickers)

    segment_set: set[str] | None = None
    if args.segment:
        segments = load_segments()
        if not segments:
            print("Error: segments.json not found. Run --fetch-segments first.")
            sys.exit(1)
        target_segments = args.segment
        segment_set = {
            code for code, seg in segments.items()
            if any(t in seg for t in target_segments)
        }

    if ticker_set and segment_set is not None:
        # AND: tickers that are also in the requested segments
        return ticker_set & segment_set
    elif ticker_set:
        return ticker_set
    elif segment_set is not None:
        return segment_set
    else:
        return None


def main():
    parser = argparse.ArgumentParser(description="TDNet Disclosure PDF Viewer")
    parser.add_argument("--date", default=date.today().strftime("%Y%m%d"),
                        help="Date to fetch (yyyymmdd)")
    parser.add_argument("--port", type=int, default=DEFAULT_PORT)
    parser.add_argument("--ticker-csv", help="Path to ticker CSV file")
    parser.add_argument("-t", "--tickers", nargs="*",
                        help="Ticker codes to filter")
    parser.add_argument("--segment", nargs="*",
                        help="Market segments to filter (e.g. プライム)")
    parser.add_argument("--fetch-segments", action="store_true",
                        help="Fetch segment data from JPX and save locally")
    parser.add_argument("--period", choices=VALID_PERIODS, default="day",
                        help="Period to fetch (day/week/month/year)")
    parser.add_argument("--all", action="store_true",
                        help="Show all disclosures (no filter)")
    args = parser.parse_args()

    if args.fetch_segments:
        print("Fetching segment data from JPX...")
        data = fetch_jpx_segments()
        save_segments(data)
        print(f"Saved {len(data)} entries to segments.json")
        return

    ticker_filter = build_ticker_filter(args)
    if ticker_filter is not None:
        set_ticker_filter(ticker_filter)
        print(f"Filter active: {len(ticker_filter)} tickers")

    if not args.all:
        segments = load_segments()
        if segments:
            etf_tickers = {code for code, seg in segments.items() if "ETF" in seg}
            if etf_tickers:
                set_exclude_tickers(etf_tickers)
                print(f"Excluding {len(etf_tickers)} ETF/ETN tickers")

    # Pre-fetch disclosure lists for the specified period
    dates = date_range(args.date, args.period)
    total = 0
    for i, d in enumerate(dates):
        print(f"\rFetching disclosures... ({i + 1}/{len(dates)})", end="", flush=True)
        items = fetch_disclosures(d)
        cache_set(d, items)
        total += len(items)
    print(f"\rFetched {total} items over {len(dates)} day(s).     ")

    kill_listeners(args.port)

    url = f"http://localhost:{args.port}"
    print(f"Starting server at {url}")
    webbrowser.open(url)
    app.run(host="127.0.0.1", port=args.port, debug=False)


if __name__ == "__main__":
    main()
