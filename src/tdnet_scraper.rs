use anyhow::Result;
use scraper::{ElementRef, Html, Selector};

use crate::config::TDNET_BASE_URL;
use crate::disclosure::Disclosure;
use crate::http_client::TdnetClient;

pub async fn fetch_disclosures(client: &TdnetClient, date: &str) -> Result<Vec<Disclosure>> {
    let first_url = list_url(1, date);
    let first_html = client.get_text(&first_url, 15).await?;

    let (max_page, mut all_items) = {
        let first_doc = Html::parse_document(&first_html);
        let table_selector = Selector::parse("table").expect("valid table selector");
        let tables: Vec<_> = first_doc.select(&table_selector).collect();
        if tables.len() < 2 {
            return Ok(Vec::new());
        }

        let div_selector = Selector::parse("div").expect("valid div selector");
        if !tables[1]
            .select(&div_selector)
            .any(|div| has_class(&div, "kaijiSum"))
        {
            return Ok(Vec::new());
        }

        let mut max_page = 1usize;
        for div in tables[1].select(&div_selector) {
            if !has_class(&div, "pager-M") {
                continue;
            }
            let text = text_of(&div);
            if let Ok(page) = text.parse::<usize>() {
                max_page = max_page.max(page);
            }
        }

        (max_page, parse_page(&first_html))
    };

    for page in 2..=max_page {
        let html = client.get_text(&list_url(page, date), 15).await?;
        all_items.extend(parse_page(&html));
    }
    Ok(all_items)
}

pub fn parse_page(html: &str) -> Vec<Disclosure> {
    let doc = Html::parse_document(html);
    let table_selector = Selector::parse("table").expect("valid table selector");
    let row_selector = Selector::parse("tr").expect("valid row selector");
    let cell_selector = Selector::parse("td").expect("valid cell selector");
    let link_selector = Selector::parse("a").expect("valid link selector");

    let tables: Vec<_> = doc.select(&table_selector).collect();
    let Some(data_table) = tables.iter().find(|table| {
        table.select(&cell_selector).any(|cell| {
            let classes: Vec<_> = cell.value().classes().collect();
            classes.contains(&"kjTitle")
        })
    }) else {
        return Vec::new();
    };

    let mut items = Vec::new();
    for row in data_table.select(&row_selector) {
        let mut item = Disclosure::default();
        for cell in row.select(&cell_selector) {
            let classes: Vec<_> = cell.value().classes().collect();
            if classes.contains(&"kjTime") {
                item.time = text_of(&cell);
            } else if classes.contains(&"kjCode") {
                item.code = text_of(&cell);
            } else if classes.contains(&"kjName") {
                item.name = text_of(&cell);
            } else if classes.contains(&"kjTitle") {
                item.title = text_of(&cell);
                if let Some(href) = first_href(&cell, &link_selector) {
                    item.pdf = href.rsplit('/').next().unwrap_or(&href).to_string();
                }
            } else if classes.contains(&"kjXbrl")
                && let Some(href) = first_href(&cell, &link_selector)
            {
                item.xbrl = Some(href);
            }
        }
        if !item.pdf.is_empty() {
            items.push(item);
        }
    }
    items
}

fn list_url(page: usize, date: &str) -> String {
    format!("{TDNET_BASE_URL}I_list_{page:03}_{date}.html")
}

fn first_href(cell: &ElementRef<'_>, link_selector: &Selector) -> Option<String> {
    cell.select(link_selector)
        .next()
        .and_then(|a| a.value().attr("href"))
        .map(ToOwned::to_owned)
}

fn text_of(element: &ElementRef<'_>) -> String {
    element.text().collect::<String>().trim().to_string()
}

fn has_class(element: &ElementRef<'_>, class_name: &str) -> bool {
    element.value().classes().any(|class| class == class_name)
}

#[cfg(test)]
mod tests {
    use super::parse_page;

    const MOCK_HTML: &str = r#"
<html><body>
<table><tr><td></td></tr></table>
<table><tr><td></td></tr></table>
<table><tr><td></td></tr></table>
<table>
  <tr>
    <td class="td kjTime">15:00</td>
    <td class="td kjCode">12340</td>
    <td class="td kjName">テスト株式会社</td>
    <td class="td kjTitle"><a href="I_test001.pdf">決算短信</a></td>
    <td class="td kjXbrl"><a href="xbrl_test.zip">XBRL</a></td>
  </tr>
  <tr>
    <td class="td kjTime">16:00</td>
    <td class="td kjCode">56780</td>
    <td class="td kjName">サンプル株式会社</td>
    <td class="td kjTitle"><a href="https://example.com/path/I_test002.pdf">適時開示</a></td>
    <td class="td kjXbrl"></td>
  </tr>
  <tr>
    <td class="td kjTime">17:00</td>
    <td class="td kjCode">99990</td>
    <td class="td kjName">空PDF株式会社</td>
    <td class="td kjTitle">タイトルのみ</td>
  </tr>
</table>
</body></html>
"#;

    #[test]
    fn parses_disclosure_rows() {
        let items = parse_page(MOCK_HTML);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].time, "15:00");
        assert_eq!(items[0].code, "12340");
        assert_eq!(items[0].name, "テスト株式会社");
        assert_eq!(items[0].title, "決算短信");
        assert_eq!(items[0].pdf, "I_test001.pdf");
        assert_eq!(items[0].xbrl.as_deref(), Some("xbrl_test.zip"));
        assert_eq!(items[1].pdf, "I_test002.pdf");
    }

    #[test]
    fn excludes_rows_without_pdf() {
        let codes: Vec<_> = parse_page(MOCK_HTML)
            .into_iter()
            .map(|item| item.code)
            .collect();
        assert!(!codes.iter().any(|code| code == "99990"));
    }

    #[test]
    fn handles_insufficient_tables() {
        assert!(parse_page("<html><body><table></table></body></html>").is_empty());
    }
}
