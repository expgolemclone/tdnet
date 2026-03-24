import argparse
import webbrowser
from datetime import date

from config import DEFAULT_PORT
from server import app


def main():
    parser = argparse.ArgumentParser(description="TDNet Disclosure PDF Viewer")
    parser.add_argument("--date", default=date.today().strftime("%Y%m%d"),
                        help="Date to fetch (yyyymmdd)")
    parser.add_argument("--port", type=int, default=DEFAULT_PORT)
    args = parser.parse_args()

    # Pre-fetch the disclosure list for the specified date
    from scraper import fetch_disclosures
    from server import _cache
    print(f"Fetching disclosures for {args.date}...")
    _cache[args.date] = fetch_disclosures(args.date)
    print(f"Found {len(_cache[args.date])} items.")

    url = f"http://localhost:{args.port}"
    print(f"Starting server at {url}")
    webbrowser.open(url)
    app.run(host="127.0.0.1", port=args.port, debug=False)


if __name__ == "__main__":
    main()
