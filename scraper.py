from bs4 import BeautifulSoup

from config import TDNET_BASE_URL, TDNET_LIST_URL
from net import session


def fetch_disclosures(date: str) -> list[dict]:
    """Fetch all disclosures for a given date (yyyymmdd format)."""
    first_url = TDNET_LIST_URL.format(page=1, date=date)
    resp = session.get(first_url, timeout=15)
    resp.encoding = "utf-8"
    soup = BeautifulSoup(resp.text, "html.parser")

    tables = soup.find_all("table")
    if len(tables) < 2:
        return []

    # Determine total pages from pagination
    info_table = tables[1]
    kaiji_sum = info_table.find_all("div", {"class": "kaijiSum"})
    if not kaiji_sum:
        return []

    pager_items = info_table.find_all("div", {"class": "pager-M"})
    page_nums = []
    for d in pager_items:
        if d.string and d.string.strip().isdigit():
            page_nums.append(int(d.string.strip()))
    max_page = max(page_nums, default=1)

    # Scrape each page
    all_items = []
    for page in range(1, max_page + 1):
        if page == 1:
            page_soup = soup
        else:
            url = TDNET_LIST_URL.format(page=page, date=date)
            r = session.get(url, timeout=15)
            r.encoding = "utf-8"
            page_soup = BeautifulSoup(r.text, "html.parser")

        items = _parse_page(page_soup)
        all_items.extend(items)

    return all_items


def _parse_page(soup: BeautifulSoup) -> list[dict]:
    tables = soup.find_all("table")
    if len(tables) < 4:
        return []

    data_table = tables[3]
    rows = data_table.find_all("tr")
    items = []

    for tr in rows:
        tds = tr.find_all("td")
        item = {}
        for td in tds:
            classes = td.get("class", [])
            if len(classes) < 2:
                continue
            cls = classes[1]
            if cls == "kjTime":
                item["time"] = td.text.strip()
            elif cls == "kjCode":
                item["code"] = td.text.strip()
            elif cls == "kjName":
                item["name"] = td.text.strip()
            elif cls == "kjTitle":
                item["title"] = td.text.strip()
                if td.a and td.a.get("href"):
                    item["pdf"] = td.a["href"].split("/")[-1]
            elif cls == "kjXbrl":
                if td.a and td.a.get("href"):
                    item["xbrl"] = td.a["href"]

        if item.get("pdf"):
            items.append(item)

    return items
