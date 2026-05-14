from pathlib import Path

TDNET_BASE_URL = "https://www.release.tdnet.info/inbs/"
TDNET_EXPECTED_HOST = "www.release.tdnet.info"
TDNET_LIST_URL = TDNET_BASE_URL + "I_list_{page:03d}_{date}.html"

JPX_DATA_URL = (
    "https://www.jpx.co.jp/markets/statistics-equities/misc/"
    "tvdivq0000001vg2-att/data_j.xls"
)

DEFAULT_PORT = 8080
CACHE_MAX_SIZE = 10
PROJECT_ROOT = Path(__file__).resolve().parent.parent
SEGMENTS_PATH = PROJECT_ROOT / "segments.json"

RETRY_TOTAL = 3
RETRY_BACKOFF = 0.5

VALID_PERIODS = ("day", "week", "month", "year")
PERIOD_DAYS = {
    "day": 1,
    "week": 5,
    "month": 22,
    "year": 250,
}
