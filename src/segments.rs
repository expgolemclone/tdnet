use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Cursor;
use std::path::Path;

use anyhow::{Context, Result};
use calamine::{Data, Reader, Xls};

use crate::config::{JPX_DATA_URL, segments_path};
use crate::http_client::TdnetClient;

pub type Segments = HashMap<String, String>;

pub async fn fetch_jpx_segments(client: &TdnetClient) -> Result<Segments> {
    let bytes = client.get_bytes_checked(JPX_DATA_URL, 30).await?;
    let mut workbook = Xls::new(Cursor::new(bytes.to_vec())).context("failed to open JPX XLS")?;
    let range = workbook
        .worksheet_range_at(0)
        .context("JPX XLS has no first sheet")?
        .context("failed to read first JPX sheet")?;

    let mut rows = range.rows();
    let headers = rows
        .next()
        .context("JPX XLS has no header row")?
        .iter()
        .map(cell_to_string)
        .collect::<Vec<_>>();

    let code_col = headers.iter().position(|header| header == "コード");
    let segment_col = headers.iter().position(|header| header == "市場・商品区分");
    let (code_col, segment_col) = match (code_col, segment_col) {
        (Some(code_col), Some(segment_col)) => (code_col, segment_col),
        _ => anyhow::bail!("Expected columns not found in XLS. Headers: {headers:?}"),
    };

    let mut segments = Segments::new();
    for row in rows {
        let Some(raw_code) = row.get(code_col) else {
            continue;
        };
        let code = code_cell_to_string(raw_code);
        if code.is_empty() {
            continue;
        }
        let segment = row.get(segment_col).map(cell_to_string).unwrap_or_default();
        if !segment.is_empty() {
            segments.insert(code, segment);
        }
    }
    Ok(segments)
}

pub fn save_segments(data: &Segments, path: &Path) -> Result<()> {
    let ordered = data
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    let json = serde_json::to_string_pretty(&ordered)?;
    fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}

pub fn load_segments(path: &Path) -> Result<Segments> {
    if !path.exists() {
        return Ok(Segments::new());
    }
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn load_default_segments() -> Result<Segments> {
    load_segments(&segments_path())
}

fn cell_to_string(cell: &Data) -> String {
    cell.to_string().trim().to_string()
}

fn code_cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Float(value) if value.fract() == 0.0 => format!("{value:.0}"),
        Data::Int(value) => value.to_string(),
        _ => cell_to_string(cell),
    }
}
