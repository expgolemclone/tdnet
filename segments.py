import json
from pathlib import Path

import requests
import xlrd

from config import JPX_DATA_URL, SEGMENTS_PATH


def fetch_jpx_segments() -> dict[str, str]:
    """Download JPX listed-company XLS and return {ticker: segment} mapping."""
    resp = requests.get(JPX_DATA_URL, timeout=30)
    resp.raise_for_status()
    book = xlrd.open_workbook(file_contents=resp.content)
    sheet = book.sheet_by_index(0)

    # Find column indices from header row
    headers = [str(sheet.cell_value(0, c)).strip() for c in range(sheet.ncols)]
    code_col = None
    segment_col = None
    for i, h in enumerate(headers):
        if h == "コード":
            code_col = i
        if h == "市場・商品区分":
            segment_col = i
    if code_col is None or segment_col is None:
        raise ValueError(f"Expected columns not found in XLS. Headers: {headers}")

    segments: dict[str, str] = {}
    for row in range(1, sheet.nrows):
        raw_code = sheet.cell_value(row, code_col)
        if isinstance(raw_code, float):
            code = str(int(raw_code))
        else:
            code = str(raw_code).strip()
        if not code:
            continue
        segment = str(sheet.cell_value(row, segment_col)).strip()
        if segment:
            segments[code] = segment

    return segments


def save_segments(data: dict[str, str], path: Path = SEGMENTS_PATH) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")


def load_segments(path: Path = SEGMENTS_PATH) -> dict[str, str]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))
