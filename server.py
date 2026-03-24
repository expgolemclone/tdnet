import requests as req
from flask import Flask, jsonify, request, Response, send_from_directory

from config import TDNET_BASE_URL
from scraper import fetch_disclosures

app = Flask(__name__, static_folder="static")

# In-memory cache: date -> list of disclosures
_cache: dict[str, list[dict]] = {}


@app.route("/")
def index():
    return send_from_directory("static", "viewer.html")


@app.route("/api/list")
def api_list():
    date = request.args.get("date", "")
    if not date or len(date) != 8:
        return jsonify({"error": "date param required (yyyymmdd)"}), 400

    if date not in _cache:
        _cache[date] = fetch_disclosures(date)

    return jsonify(_cache[date])


@app.route("/api/pdf/<filename>")
def proxy_pdf(filename):
    if not filename.endswith(".pdf"):
        return "Not found", 404

    url = TDNET_BASE_URL + filename
    resp = req.get(url, timeout=30, stream=True)
    if resp.status_code != 200:
        return "Not found", 404

    return Response(
        resp.iter_content(chunk_size=8192),
        content_type="application/pdf",
        headers={"Cache-Control": "public, max-age=86400"},
    )
