from bs4 import BeautifulSoup

from scraper import _parse_page


MOCK_HTML = """
<html><body>
<table></table>
<table></table>
<table></table>
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
"""


class TestParsePage:
    def test_basic_parsing(self):
        soup = BeautifulSoup(MOCK_HTML, "html.parser")
        items = _parse_page(soup)
        assert len(items) == 2  # 3rd row has no PDF link

    def test_fields(self):
        soup = BeautifulSoup(MOCK_HTML, "html.parser")
        items = _parse_page(soup)
        first = items[0]
        assert first["time"] == "15:00"
        assert first["code"] == "12340"
        assert first["name"] == "テスト株式会社"
        assert first["title"] == "決算短信"
        assert first["pdf"] == "I_test001.pdf"
        assert first["xbrl"] == "xbrl_test.zip"

    def test_full_url_extracts_filename(self):
        soup = BeautifulSoup(MOCK_HTML, "html.parser")
        items = _parse_page(soup)
        second = items[1]
        assert second["pdf"] == "I_test002.pdf"

    def test_no_pdf_excluded(self):
        soup = BeautifulSoup(MOCK_HTML, "html.parser")
        items = _parse_page(soup)
        codes = [i["code"] for i in items]
        assert "99990" not in codes

    def test_insufficient_tables(self):
        html = "<html><body><table></table></body></html>"
        soup = BeautifulSoup(html, "html.parser")
        assert _parse_page(soup) == []
