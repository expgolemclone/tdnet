from pathlib import Path

TDNET_BASE_URL = "https://www.release.tdnet.info/inbs/"
TDNET_LIST_URL = TDNET_BASE_URL + "I_list_{page:03d}_{date}.html"

JPX_DATA_URL = (
    "https://www.jpx.co.jp/markets/statistics-equities/misc/"
    "tvdivq0000001vg2-att/data_j.xls"
)

DEFAULT_PORT = 8080
CACHE_MAX_SIZE = 10
SEGMENTS_PATH = Path(__file__).parent / "segments.json"
