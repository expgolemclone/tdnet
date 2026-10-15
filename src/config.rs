use std::path::PathBuf;

pub const TDNET_BASE_URL: &str = "https://www.release.tdnet.info/inbs/";
pub const TDNET_EXPECTED_HOST: &str = "www.release.tdnet.info";
pub const JPX_DATA_URL: &str =
    "https://www.jpx.co.jp/markets/statistics-equities/misc/tvdivq0000001vg2-att/data_j.xls";

pub const DEFAULT_PORT: u16 = 8080;
pub const CACHE_MAX_SIZE: usize = 10;
pub const RETRY_TOTAL: usize = 3;
pub const RETRY_BACKOFF_MS: u64 = 500;

pub fn segments_path() -> PathBuf {
    PathBuf::from("segments.json")
}
