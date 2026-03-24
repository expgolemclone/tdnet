import argparse
import webbrowser
from datetime import date

import truststore

truststore.inject_into_ssl()

from config import DEFAULT_PORT
from server import app, cache_set
from scraper import fetch_disclosures


def main():
    parser = argparse.ArgumentParser(description="TDNet Disclosure PDF Viewer")
    parser.add_argument("--date", default=date.today().strftime("%Y%m%d"),
                        help="Date to fetch (yyyymmdd)")
    parser.add_argument("--port", type=int, default=DEFAULT_PORT)
    args = parser.parse_args()

    # Pre-fetch the disclosure list for the specified date
    print(f"Fetching disclosures for {args.date}...")
    items = fetch_disclosures(args.date)
    cache_set(args.date, items)
    print(f"Found {len(items)} items.")

    url = f"http://localhost:{args.port}"
    print(f"Starting server at {url}")
    webbrowser.open(url)
    app.run(host="127.0.0.1", port=args.port, debug=False)


if __name__ == "__main__":
    main()
