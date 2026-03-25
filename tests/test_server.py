from server import date_range, cache_set, _apply_filter, _cache, _cache_lock
from config import CACHE_MAX_SIZE


class TestDateRange:
    def test_single_day(self):
        result = date_range("20260325", "day")
        assert result == ["20260325"]

    def test_skips_weekends(self):
        # 20260323 is Monday, 20260321 is Saturday, 20260322 is Sunday
        result = date_range("20260327", "week")  # Friday
        assert len(result) == 5
        for d in result:
            from datetime import date as dt_date
            parsed = dt_date(int(d[:4]), int(d[4:6]), int(d[6:8]))
            assert parsed.weekday() < 5, f"{d} is a weekend"

    def test_weekdays_only_in_week(self):
        result = date_range("20260325", "week")  # Wednesday
        assert len(result) == 5
        # All should be weekdays
        from datetime import date as dt_date
        for d in result:
            parsed = dt_date(int(d[:4]), int(d[4:6]), int(d[6:8]))
            assert parsed.weekday() < 5

    def test_dates_descending(self):
        result = date_range("20260325", "week")
        assert result[0] == "20260325"
        assert result == sorted(result, reverse=True)


class TestCacheSet:
    def setup_method(self):
        _cache.clear()

    def test_basic_set(self):
        cache_set("20260325", [{"a": 1}])
        with _cache_lock:
            assert "20260325" in _cache
            assert _cache["20260325"] == [{"a": 1}]

    def test_eviction(self):
        for i in range(CACHE_MAX_SIZE + 3):
            cache_set(f"2026010{i:02d}", [])
        with _cache_lock:
            assert len(_cache) == CACHE_MAX_SIZE


class TestApplyFilter:
    def test_no_filter(self):
        items = [{"code": "1234"}, {"code": "5678"}]
        assert _apply_filter(items) == items

    def test_ticker_filter(self):
        from server import set_ticker_filter
        try:
            set_ticker_filter({"1234"})
            items = [{"code": "12340"}, {"code": "56780"}]
            result = _apply_filter(items)
            assert len(result) == 1
            assert result[0]["code"] == "12340"
        finally:
            set_ticker_filter(None)

    def test_exclude_tickers(self):
        from server import set_exclude_tickers
        try:
            set_exclude_tickers({"5678"})
            items = [{"code": "12340"}, {"code": "56780"}]
            result = _apply_filter(items)
            assert len(result) == 1
            assert result[0]["code"] == "12340"
        finally:
            set_exclude_tickers(set())
