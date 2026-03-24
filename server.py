import re
from collections import OrderedDict

import requests
from flask import Flask, jsonify, request, Response, send_from_directory

from config import TDNET_BASE_URL, CACHE_MAX_SIZE
from scraper import fetch_disclosures

app = Flask(__name__, static_folder="static")

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
    if _ticker_filter is None and not _exclude_tickers:
        return items
    result = items
    if _ticker_filter is not None:
        result = [item for item in result if item.get("code", "")[:4] in _ticker_filter]
    if _exclude_tickers:
        result = [item for item in result if item.get("code", "")[:4] not in _exclude_tickers]
    return result


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
    date = request.args.get("date", "")
    if not date or len(date) != 8:
        return jsonify({"error": "date param required (yyyymmdd)"}), 400

    if date not in _cache:
        cache_set(date, fetch_disclosures(date))
    else:
        _cache.move_to_end(date)

    return jsonify(_apply_filter(_cache[date]))


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
