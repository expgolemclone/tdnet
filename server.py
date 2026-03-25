import re
from collections import OrderedDict
from datetime import date, timedelta

import requests
from flask import Flask, jsonify, request, Response, send_from_directory

from config import TDNET_BASE_URL, CACHE_MAX_SIZE, VALID_PERIODS, PERIOD_DAYS
from scraper import fetch_disclosures

app = Flask(__name__, static_folder="static")


@app.after_request
def no_cache_static(response):
    if response.content_type and ("javascript" in response.content_type or "css" in response.content_type):
        response.headers["Cache-Control"] = "no-store"
    return response


# LRU cache: date -> list of disclosures
_cache: OrderedDict[str, list[dict]] = OrderedDict()

_SAFE_PDF_NAME = re.compile(r"^[A-Za-z0-9_-]+\.pdf$")

_ticker_filter: set[str] | None = None
_exclude_tickers: set[str] = set()


def set_ticker_filter(tickers: set[str] | None) -> None:
    global _ticker_filter
    _ticker_filter = tickers


def set_exclude_tickers(tickers: set[str]) -> None:
    global _exclude_tickers
    _exclude_tickers = tickers


def _apply_filter(items: list[dict]) -> list[dict]:
    print(f"[filter] ticker={('None' if _ticker_filter is None else len(_ticker_filter))}, exclude={len(_exclude_tickers)}, items={len(items)}")
    if _ticker_filter is None and not _exclude_tickers:
        return items
    result = items
    if _ticker_filter is not None:
        result = [item for item in result if item.get("code", "")[:4] in _ticker_filter]
    if _exclude_tickers:
        result = [item for item in result if item.get("code", "")[:4] not in _exclude_tickers]
    return result


def date_range(end_date_str: str, period: str) -> list[str]:
    """Return list of yyyymmdd strings for the given period ending on end_date_str."""
    end = date(int(end_date_str[:4]), int(end_date_str[4:6]), int(end_date_str[6:8]))
    days = PERIOD_DAYS.get(period, 1)
    return [(end - timedelta(days=i)).strftime("%Y%m%d") for i in range(days)]


def cache_set(date: str, items: list[dict]) -> None:
    _cache[date] = items
    _cache.move_to_end(date)
    while len(_cache) > CACHE_MAX_SIZE:
        _cache.popitem(last=False)


@app.route("/")
def index():
    return send_from_directory("static", "viewer.html")


@app.route("/api/list")
def api_list():
    end_date = request.args.get("date", "")
    if not end_date or len(end_date) != 8:
        return jsonify({"error": "date param required (yyyymmdd)"}), 400

    period = request.args.get("period", "day")
    if period not in VALID_PERIODS:
        return jsonify({"error": f"invalid period (choose from {VALID_PERIODS})"}), 400

    dates = date_range(end_date, period)
    all_items: list[dict] = []
    for d in dates:
        if d not in _cache:
            cache_set(d, fetch_disclosures(d))
        else:
            _cache.move_to_end(d)
        all_items.extend(_cache[d])

    return jsonify(_apply_filter(all_items))


@app.route("/api/pdf/<filename>")
def proxy_pdf(filename):
    if not _SAFE_PDF_NAME.match(filename):
        return "Not found", 404

    url = TDNET_BASE_URL + filename
    resp = requests.get(url, timeout=30, stream=True)
    if resp.status_code != 200:
        resp.close()
        return "Not found", 404

    def generate():
        try:
            yield from resp.iter_content(chunk_size=8192)
        finally:
            resp.close()

    return Response(
        generate(),
        content_type="application/pdf",
        headers={"Cache-Control": "public, max-age=86400"},
    )
