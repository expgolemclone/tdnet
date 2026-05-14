pub const VIEWER_HTML: &str = r###"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="color-scheme" content="dark">
    <title>TDNet Viewer</title>
    <link rel="stylesheet" href="/static/style.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.min.js"></script>
</head>
<body>
    <header id="top-bar">
        <div id="date-control">
            <button id="prev-date" title="前日">&larr;</button>
            <input type="date" id="date-picker">
            <button id="next-date" title="翌日">&rarr;</button>
            <select id="period-select">
                <option value="day">日</option>
                <option value="week">週</option>
                <option value="month">月</option>
                <option value="year">年</option>
            </select>
        </div>
        <div id="file-info">
            <span id="file-index">-/-</span>
            <span id="file-time"></span>
            <span id="file-code"></span>
            <span id="file-name"></span>
            <span id="file-title"></span>
        </div>
    </header>

    <main id="viewer">
        <div id="loading">読み込み中...</div>
        <canvas id="pdf-canvas"></canvas>
        <div id="empty-msg" style="display:none;">開示情報がありません</div>
    </main>

    <footer id="bottom-bar">
        <span id="page-info">- / -</span>
        <span id="keyhints">
            <kbd>j</kbd><kbd>k</kbd> ページ送り
            <kbd>h</kbd><kbd>l</kbd> ファイル切替
            <kbd>o</kbd> 番号ジャンプ
            <kbd>gg</kbd><kbd>G</kbd> 先頭/末尾
            <kbd>yy</kbd> コピー
            <kbd>:tree</kbd> 一覧
        </span>
    </footer>

    <div id="copy-toast"></div>

    <div id="goto-overlay" style="display:none;">
        <input type="text" id="goto-input" placeholder="#">
    </div>

    <div id="cmd-overlay" style="display:none;">
        <span id="cmd-prefix">:</span>
        <input type="text" id="cmd-input">
    </div>

    <div id="tree-overlay" style="display:none;">
        <div id="tree-header">
            <span id="tree-title">開示一覧</span>
            <span id="tree-count"></span>
            <div id="tree-filter-box" style="display:none;">
                <span>/</span>
                <input type="text" id="tree-filter-input">
            </div>
        </div>
        <div id="tree-body"></div>
    </div>

    <script src="/static/viewer.js"></script>
</body>
</html>
"###;

pub const STYLE_CSS: &str = r###":root {
    color-scheme: dark;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    background: #1a1a2e;
    color: #e0e0e0;
    font-family: "Segoe UI", "Meiryo", sans-serif;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

header#top-bar {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.4rem 1rem;
    background: #16213e;
    border-bottom: 1px solid #0f3460;
    flex-shrink: 0;
    min-height: 2.4rem;
}

#date-control {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    flex-shrink: 0;
}

#date-control button {
    background: #0f3460;
    color: #e0e0e0;
    border: none;
    padding: 0.2rem 0.5rem;
    cursor: pointer;
    border-radius: 3px;
    font-size: 0.9rem;
}

#date-control button:hover {
    background: #533483;
}

#date-picker,
#period-select {
    background: #0f3460;
    color: #e0e0e0;
    border: 1px solid #533483;
    padding: 0.15rem 0.4rem;
    border-radius: 3px;
    font-size: 0.85rem;
    color-scheme: dark;
}

#file-info {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    overflow: hidden;
    white-space: nowrap;
    font-size: 0.9rem;
}

#file-index {
    color: #e94560;
    font-weight: bold;
    flex-shrink: 0;
}

#file-time {
    color: #999;
    flex-shrink: 0;
}

#file-code {
    color: #53cfcf;
    flex-shrink: 0;
}

#file-name {
    color: #aaa;
    flex-shrink: 0;
}

#file-title {
    color: #e0e0e0;
    overflow: hidden;
    text-overflow: ellipsis;
}

main#viewer {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    overflow: auto;
    background: #0a0a1a;
    position: relative;
}

#pdf-canvas {
    max-width: 100%;
    height: auto;
    filter: invert(1) hue-rotate(180deg);
}

#loading,
#empty-msg {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    color: #666;
    font-size: 1.2rem;
}

#loading {
    display: none;
}

footer#bottom-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.3rem 1rem;
    background: #16213e;
    border-top: 1px solid #0f3460;
    flex-shrink: 0;
    font-size: 0.85rem;
}

#page-info {
    color: #e94560;
    font-weight: bold;
}

#keyhints {
    color: #666;
}

kbd {
    background: #0f3460;
    color: #53cfcf;
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.8rem;
    margin: 0 0.1rem;
}

#goto-overlay {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: #16213e;
    border: 1px solid #533483;
    border-radius: 6px;
    padding: 1rem;
    z-index: 100;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.6);
}

#goto-input {
    background: #0f3460;
    color: #e0e0e0;
    border: 1px solid #533483;
    padding: 0.4rem 0.6rem;
    border-radius: 4px;
    font-size: 1.2rem;
    width: 6rem;
    text-align: center;
    outline: none;
}

#goto-input:focus {
    border-color: #e94560;
}

#copy-toast {
    position: fixed;
    bottom: 3rem;
    left: 50%;
    transform: translateX(-50%);
    background: #533483;
    color: #e0e0e0;
    padding: 0.4rem 1.2rem;
    border-radius: 4px;
    font-size: 0.9rem;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.3s;
    z-index: 100;
}

#copy-toast.show {
    opacity: 1;
}

#cmd-overlay {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    background: #16213e;
    border-top: 1px solid #0f3460;
    display: flex;
    align-items: center;
    padding: 0.2rem 0.5rem;
    z-index: 200;
    font-family: monospace;
    font-size: 0.95rem;
}

#cmd-prefix {
    color: #e94560;
    margin-right: 0.2rem;
}

#cmd-input {
    background: transparent;
    color: #e0e0e0;
    border: none;
    outline: none;
    font-family: monospace;
    font-size: 0.95rem;
    flex: 1;
}

#tree-overlay {
    position: fixed;
    inset: 0;
    background: rgba(10, 10, 26, 0.97);
    z-index: 150;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

#tree-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.6rem 1rem;
    background: #16213e;
    border-bottom: 1px solid #0f3460;
    flex-shrink: 0;
}

#tree-title {
    color: #e94560;
    font-weight: bold;
    font-size: 1rem;
}

#tree-count {
    color: #666;
    font-size: 0.85rem;
}

#tree-filter-box {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    margin-left: auto;
    color: #e94560;
    font-family: monospace;
}

#tree-filter-input {
    background: #0f3460;
    color: #e0e0e0;
    border: 1px solid #533483;
    padding: 0.15rem 0.4rem;
    border-radius: 3px;
    font-size: 0.85rem;
    outline: none;
    width: 16rem;
}

#tree-filter-input:focus {
    border-color: #e94560;
}

#tree-body {
    flex: 1;
    overflow-y: auto;
    padding: 0.3rem 0;
}

.tree-row {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    padding: 0.25rem 1rem;
    cursor: pointer;
    font-size: 0.85rem;
    white-space: nowrap;
}

.tree-row:hover {
    background: rgba(83, 52, 131, 0.3);
}

.tree-row.active {
    background: rgba(233, 69, 96, 0.15);
}

.tree-row.selected {
    background: #533483;
}

.tree-row .tr-idx {
    color: #666;
    width: 3rem;
    text-align: right;
    flex-shrink: 0;
}

.tree-row .tr-time {
    color: #999;
    width: 3.5rem;
    flex-shrink: 0;
}

.tree-row .tr-code {
    color: #53cfcf;
    width: 4rem;
    flex-shrink: 0;
}

.tree-row .tr-name {
    color: #aaa;
    width: 10rem;
    flex-shrink: 0;
    overflow: hidden;
    text-overflow: ellipsis;
}

.tree-row .tr-title {
    color: #e0e0e0;
    overflow: hidden;
    text-overflow: ellipsis;
}
"###;

pub const VIEWER_JS: &str = r###"pdfjsLib.GlobalWorkerOptions.workerSrc =
    "https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js";

const $ = (sel) => document.querySelector(sel);

const state = {
    files: [],
    fileIdx: 0,
    pageNum: 1,
    pageCount: 0,
    pdfDoc: null,
    rendering: false,
    pendingRender: false,
    date: "",
    period: "day",
    loadId: 0,
    dateLoadId: 0,
};

const canvas = $("#pdf-canvas");
const ctx = canvas.getContext("2d");
const loading = $("#loading");
const emptyMsg = $("#empty-msg");
const datePicker = $("#date-picker");
const periodSelect = $("#period-select");

function dateStr(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${y}${m}${day}`;
}

function dateISO(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${y}-${m}-${day}`;
}

const PERIOD_SHIFT = { day: 1, week: 7, month: 31, year: 365 };

function shiftDate(direction) {
    const d = new Date(datePicker.value);
    const delta = direction * (PERIOD_SHIFT[state.period] || 1);
    d.setDate(d.getDate() + delta);
    datePicker.value = dateISO(d);
    loadDate(dateStr(d));
}

async function fetchList(date, period) {
    const resp = await fetch(`/api/list?date=${date}&period=${period}`);
    if (!resp.ok) return [];
    return resp.json();
}

let currentLoadTask = null;

async function loadPdf(filename) {
    window.scrollTo(0, 0);

    if (currentLoadTask) {
        currentLoadTask.destroy();
        currentLoadTask = null;
    }

    const myId = ++state.loadId;

    loading.style.display = "block";
    canvas.style.display = "none";
    updateUI();

    if (state.pdfDoc) {
        state.pdfDoc.destroy();
        state.pdfDoc = null;
    }

    const url = `/api/pdf/${filename}`;
    const task = pdfjsLib.getDocument(url);
    currentLoadTask = task;

    let doc;
    try {
        doc = await task.promise;
    } catch (e) {
        if (myId !== state.loadId) return;
        console.error("PDF load failed:", e);
        loading.style.display = "none";
        return;
    }

    if (myId !== state.loadId) {
        doc.destroy();
        return;
    }

    currentLoadTask = null;
    state.pdfDoc = doc;
    state.pageCount = doc.numPages;
    state.pageNum = 1;

    loading.style.display = "none";
    canvas.style.display = "block";

    state.rendering = false;
    state.pendingRender = false;
    await renderPage();
}

async function renderPage() {
    if (!state.pdfDoc) return;
    if (state.rendering) {
        state.pendingRender = true;
        return;
    }
    state.rendering = true;

    try {
        const page = await state.pdfDoc.getPage(state.pageNum);
        const viewerWidth = $("#viewer").clientWidth - 16;
        const unscaled = page.getViewport({ scale: 1 });
        const scale = viewerWidth / unscaled.width;
        const viewport = page.getViewport({ scale });

        canvas.height = viewport.height;
        canvas.width = viewport.width;

        await page.render({ canvasContext: ctx, viewport }).promise;
    } catch (e) {
        console.error("Render failed:", e);
    }

    state.rendering = false;
    updateUI();

    if (state.pendingRender) {
        state.pendingRender = false;
        await renderPage();
    }
}

function nextPage() {
    if (state.pageNum < state.pageCount) {
        state.pageNum++;
        renderPage();
    }
}

function prevPage() {
    if (state.pageNum > 1) {
        state.pageNum--;
        renderPage();
    }
}

function nextFile() {
    if (state.fileIdx < state.files.length - 1) {
        state.fileIdx++;
        loadPdf(state.files[state.fileIdx].pdf);
    }
}

function prevFile() {
    if (state.fileIdx > 0) {
        state.fileIdx--;
        loadPdf(state.files[state.fileIdx].pdf);
    }
}

function updateUI() {
    const f = state.files[state.fileIdx];
    if (f) {
        $("#file-index").textContent = `${state.fileIdx + 1}/${state.files.length}`;
        $("#file-time").textContent = f.time || "";
        $("#file-code").textContent = f.code || "";
        $("#file-name").textContent = f.name || "";
        $("#file-title").textContent = f.title || "";
    } else {
        $("#file-index").textContent = "-/-";
        $("#file-time").textContent = "";
        $("#file-code").textContent = "";
        $("#file-name").textContent = "";
        $("#file-title").textContent = "";
    }
    $("#page-info").textContent = state.pageCount
        ? `${state.pageNum} / ${state.pageCount}`
        : "- / -";
}

async function loadDate(date) {
    const myId = ++state.dateLoadId;

    state.date = date;
    state.files = [];
    state.fileIdx = 0;
    canvas.style.display = "none";
    emptyMsg.style.display = "none";
    loading.style.display = "block";

    const items = await fetchList(date, state.period);

    if (myId !== state.dateLoadId) return;

    state.files = items;

    if (items.length === 0) {
        loading.style.display = "none";
        emptyMsg.style.display = "block";
        updateUI();
        return;
    }

    await loadPdf(items[0].pdf);
}

const gotoOverlay = $("#goto-overlay");
const gotoInput = $("#goto-input");

function showGoto() {
    gotoInput.value = "";
    gotoOverlay.style.display = "block";
    gotoInput.focus();
}

function hideGoto() {
    gotoOverlay.style.display = "none";
}

function gotoFile(n) {
    if (n < 1 || n > state.files.length) return;
    state.fileIdx = n - 1;
    loadPdf(state.files[state.fileIdx].pdf);
}

gotoInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
        const n = parseInt(gotoInput.value, 10);
        hideGoto();
        if (!isNaN(n)) gotoFile(n);
    } else if (e.key === "Escape") {
        hideGoto();
    }
});

async function getPageText(pageNum) {
    const page = await state.pdfDoc.getPage(pageNum);
    const content = await page.getTextContent();
    return content.items.map((item) => item.str).join("");
}

async function copyToClipboard() {
    if (!state.pdfDoc) return;
    try {
        const text = await getPageText(state.pageNum);
        if (text.trim()) {
            const quoted = text.split("\n").map(l => "> " + l).join("\n");
            await navigator.clipboard.writeText(quoted);
            showToast("引用としてコピーしました");
            return;
        }
        const blob = await new Promise((r) => canvas.toBlob(r, "image/png"));
        if (!blob) return;
        await navigator.clipboard.write([new ClipboardItem({ "image/png": blob })]);
        showToast("画像をコピーしました");
    } catch (e) {
        console.error("Clipboard write failed:", e);
        showToast("コピー失敗");
    }
}

function showToast(msg) {
    const el = $("#copy-toast");
    el.textContent = msg;
    el.classList.add("show");
    setTimeout(() => el.classList.remove("show"), 1200);
}

const cmdOverlay = $("#cmd-overlay");
const cmdInput = $("#cmd-input");

function showCmd() {
    cmdInput.value = "";
    cmdOverlay.style.display = "flex";
    cmdInput.focus();
}

function hideCmd() {
    cmdOverlay.style.display = "none";
}

function execCmd(raw) {
    const cmd = raw.trim().toLowerCase().replace(/^:/, "");
    if (cmd === "tree") {
        showTree();
    }
}

cmdInput.addEventListener("keydown", (e) => {
    e.stopPropagation();
    if (e.key === "Enter") {
        const val = cmdInput.value;
        hideCmd();
        execCmd(val);
    } else if (e.key === "Escape") {
        hideCmd();
    }
});

const treeOverlay = $("#tree-overlay");
const treeBody = $("#tree-body");
const treeCount = $("#tree-count");
const treeFilterBox = $("#tree-filter-box");
const treeFilterInput = $("#tree-filter-input");

let treeOpen = false;
let treeCursor = 0;
let treeFiltered = [];

function showTree() {
    if (state.files.length === 0) return;
    treeOpen = true;
    treeFilterBox.style.display = "none";
    treeFilterInput.value = "";
    treeFiltered = state.files.map((_, i) => i);
    treeCursor = treeFiltered.indexOf(state.fileIdx);
    if (treeCursor < 0) treeCursor = 0;
    renderTree();
    treeOverlay.style.display = "flex";
}

function hideTree() {
    treeOpen = false;
    treeOverlay.style.display = "none";
}

function renderTree() {
    treeCount.textContent = `${treeFiltered.length}/${state.files.length}`;
    treeBody.innerHTML = "";
    treeFiltered.forEach((fileIdx, vi) => {
        const f = state.files[fileIdx];
        const row = document.createElement("div");
        row.className = "tree-row";
        if (fileIdx === state.fileIdx) row.classList.add("active");
        if (vi === treeCursor) row.classList.add("selected");

        const spans = [
            ["tr-idx", String(fileIdx + 1)],
            ["tr-time", f.time || ""],
            ["tr-code", f.code || ""],
            ["tr-name", f.name || ""],
            ["tr-title", f.title || ""],
        ];
        for (const [cls, text] of spans) {
            const span = document.createElement("span");
            span.className = cls;
            span.textContent = text;
            row.appendChild(span);
        }

        row.addEventListener("click", () => {
            hideTree();
            gotoFile(fileIdx + 1);
        });
        treeBody.appendChild(row);
    });
    scrollTreeCursor();
}

function scrollTreeCursor() {
    const sel = treeBody.querySelector(".tree-row.selected");
    if (sel) sel.scrollIntoView({ block: "nearest" });
}

function treeMove(delta) {
    treeCursor = Math.max(0, Math.min(treeFiltered.length - 1, treeCursor + delta));
    updateTreeSelection();
}

function updateTreeSelection() {
    const rows = treeBody.querySelectorAll(".tree-row");
    rows.forEach((r, i) => r.classList.toggle("selected", i === treeCursor));
    scrollTreeCursor();
}

function treeSelect() {
    if (treeFiltered.length === 0) return;
    const fileIdx = treeFiltered[treeCursor];
    hideTree();
    gotoFile(fileIdx + 1);
}

function applyTreeFilter() {
    const q = treeFilterInput.value.toLowerCase();
    if (!q) {
        treeFiltered = state.files.map((_, i) => i);
    } else {
        treeFiltered = [];
        state.files.forEach((f, i) => {
            const hay = `${f.code || ""} ${f.name || ""} ${f.title || ""}`.toLowerCase();
            if (hay.includes(q)) treeFiltered.push(i);
        });
    }
    treeCursor = Math.min(treeCursor, Math.max(0, treeFiltered.length - 1));
    renderTree();
}

treeFilterInput.addEventListener("input", applyTreeFilter);
treeFilterInput.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
        treeFilterBox.style.display = "none";
        treeFilterInput.value = "";
        applyTreeFilter();
        e.stopPropagation();
    } else if (e.key === "Enter") {
        treeFilterInput.blur();
        e.stopPropagation();
    }
});

let lastKey = "";
let lastKeyTime = 0;

document.addEventListener("keydown", (e) => {
    if (treeOpen && e.target !== treeFilterInput) {
        switch (e.key) {
            case "j":
                treeMove(1);
                return;
            case "k":
                treeMove(-1);
                return;
            case "Enter":
                treeSelect();
                return;
            case "Escape":
            case "q":
                hideTree();
                return;
            case "/":
                e.preventDefault();
                treeFilterBox.style.display = "flex";
                treeFilterInput.focus();
                return;
        }
        return;
    }

    if (e.target.tagName === "INPUT") return;

    const now = Date.now();

    switch (e.key) {
        case "j":
            nextPage();
            break;
        case "k":
            prevPage();
            break;
        case "l":
            nextFile();
            break;
        case "h":
            prevFile();
            break;
        case "o":
            e.preventDefault();
            showGoto();
            break;
        case ":":
            e.preventDefault();
            showCmd();
            break;
        case "G":
            gotoFile(state.files.length);
            break;
        case "g":
            if (lastKey === "g" && now - lastKeyTime < 500) {
                gotoFile(1);
                lastKey = "";
                return;
            }
            break;
        case "y":
            if (lastKey === "y" && now - lastKeyTime < 500) {
                copyToClipboard();
                lastKey = "";
                return;
            }
            break;
    }

    lastKey = e.key;
    lastKeyTime = now;
});

$("#prev-date").addEventListener("click", () => shiftDate(-1));
$("#next-date").addEventListener("click", () => shiftDate(1));
datePicker.addEventListener("change", () => {
    loadDate(datePicker.value.replace(/-/g, ""));
});
periodSelect.addEventListener("change", () => {
    state.period = periodSelect.value;
    loadDate(datePicker.value.replace(/-/g, ""));
});

let resizeTimer = 0;
window.addEventListener("resize", () => {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => {
        if (state.pdfDoc) renderPage();
    }, 150);
});

(async function init() {
    try {
        const resp = await fetch("/api/init");
        const data = await resp.json();
        const d = data.date;
        const iso = `${d.slice(0, 4)}-${d.slice(4, 6)}-${d.slice(6, 8)}`;
        datePicker.value = iso;
        if (data.period) {
            state.period = data.period;
            periodSelect.value = data.period;
        }
        loadDate(d);
    } catch {
        const today = new Date();
        datePicker.value = dateISO(today);
        loadDate(dateStr(today));
    }
})();
"###;
