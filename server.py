import re
import threading
from collections import OrderedDict
from datetime import date, timedelta
from urllib.parse import urlparse

from flask import Flask, jsonify, request, Response, send_from_directory

from config import TDNET_BASE_URL, TDNET_EXPECTED_HOST, CACHE_MAX_SIZE, VALID_PERIODS, PERIOD_DAYS
from net import session
from scraper import fetch_disclosures

app = Flask(__name__, static_folder="static")


@app.after_request
def no_cache_static(response):
    if response.content_type and ("javascript" in response.content_type or "css" in response.content_type):
        response.headers["Cache-Control"] = "no-store"
    return response


# LRU cache: date -> list of disclosures
_cache: OrderedDict[str, list[dict]] = OrderedDict()
_cache_lock = threading.Lock()

_SAFE_PDF_NAME = re.compile(r"^[A-Za-z0-9_-]+\.pdf$")

_ticker_filter: set[str] | None = None
_exclude_tickers: set[str] = set()

_init_date: str = ""
_init_period: str = "day"


def set_init_params(init_date: str, init_period: str) -> None:
    global _init_date, _init_period
    _init_date = init_date
    _init_period = init_period


def set_ticker_filter(tickers: set[str] | None) -> None:
    global _ticker_filter
    _ticker_filter = tickers


def set_exclude_tickers(tickers: set[str]) -> None:
    global _exclude_tickers
    _exclude_tickers = tickers


def _apply_filter(items: list[dict]) -> list[dict]:
    if _ticker_filter is None and not _exclude_tickers:
        return items
    result = items
    if _ticker_filter is not None:
        result = [item for item in result if item.get("code", "")[:4] in _ticker_filter]
    if _exclude_tickers:
        result = [item for item in result if item.get("code", "")[:4] not in _exclude_tickers]
    return result


def date_range(end_date_str: str, period: str) -> list[str]:
    """Return list of yyyymmdd strings (weekdays only) for the given period."""
    end = date(int(end_date_str[:4]), int(end_date_str[4:6]), int(end_date_str[6:8]))
    target = PERIOD_DAYS.get(period, 1)
    result: list[str] = []
    offset = 0
    while len(result) < target:
        d = end - timedelta(days=offset)
        if d.weekday() < 5:  # Mon-Fri
            result.append(d.strftime("%Y%m%d"))
        offset += 1
    return result


def cache_set(date_key: str, items: list[dict]) -> None:
    with _cache_lock:
        _cache[date_key] = items
        _cache.move_to_end(date_key)
        while len(_cache) > CACHE_MAX_SIZE:
            _cache.popitem(last=False)


@app.route("/")
def index():
    return send_from_directory("static", "viewer.html")


@app.route("/api/init")
def api_init():
    return jsonify({"date": _init_date, "period": _init_period})


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
        with _cache_lock:
            if d in _cache:
                _cache.move_to_end(d)
                all_items.extend(_cache[d])
                continue
        items = fetch_disclosures(d)
        cache_set(d, items)
        all_items.extend(items)

    return jsonify(_apply_filter(all_items))


@app.route("/api/pdf/<filename>")
def proxy_pdf(filename):
    if not _SAFE_PDF_NAME.match(filename):
        return "Not found", 404

    url = TDNET_BASE_URL + filename
    if urlparse(url).hostname != TDNET_EXPECTED_HOST:
        return "Not found", 404
    resp = session.get(url, timeout=30, stream=True)
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
