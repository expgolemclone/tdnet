pdfjsLib.GlobalWorkerOptions.workerSrc =
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
    loadId: 0,
    dateLoadId: 0,
};

// --- DOM refs ---
const canvas = $("#pdf-canvas");
const ctx = canvas.getContext("2d");
const loading = $("#loading");
const emptyMsg = $("#empty-msg");
const datePicker = $("#date-picker");

// --- Date helpers ---
function dateStr(d) {
    return d.toISOString().slice(0, 10).replace(/-/g, "");
}

function dateISO(d) {
    return d.toISOString().slice(0, 10);
}

function shiftDate(delta) {
    const d = new Date(datePicker.value);
    d.setDate(d.getDate() + delta);
    datePicker.value = dateISO(d);
    loadDate(dateStr(d));
}

// --- API ---
async function fetchList(date) {
    const resp = await fetch(`/api/list?date=${date}`);
    if (!resp.ok) return [];
    return resp.json();
}

// --- PDF loading ---
let currentLoadTask = null;

async function loadPdf(filename) {
    // Cancel any in-flight PDF load
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

    // A newer load was started while we were fetching — discard this result
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

    await renderPage();
}

async function renderPage() {
    if (!state.pdfDoc) return;
    if (state.rendering) {
        state.pendingRender = true;
        return;
    }
    state.rendering = true;

    const page = await state.pdfDoc.getPage(state.pageNum);

    // Fit to viewport width
    const viewerWidth = $("#viewer").clientWidth - 16;
    const unscaled = page.getViewport({ scale: 1 });
    const scale = viewerWidth / unscaled.width;
    const viewport = page.getViewport({ scale });

    canvas.height = viewport.height;
    canvas.width = viewport.width;

    await page.render({ canvasContext: ctx, viewport }).promise;
    state.rendering = false;

    updateUI();

    if (state.pendingRender) {
        state.pendingRender = false;
        await renderPage();
    }
}

// --- Navigation ---
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

// --- UI update ---
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

// --- Date loading ---
async function loadDate(date) {
    const myId = ++state.dateLoadId;

    state.date = date;
    state.files = [];
    state.fileIdx = 0;
    canvas.style.display = "none";
    emptyMsg.style.display = "none";
    loading.style.display = "block";

    const items = await fetchList(date);

    // A newer loadDate was called while we were fetching — discard
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

// --- Goto file by number ---
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

// --- Copy canvas to clipboard ---
async function copyCanvasToClipboard() {
    if (!state.pdfDoc) return;
    try {
        const blob = await new Promise((r) => canvas.toBlob(r, "image/png"));
        if (!blob) return;
        await navigator.clipboard.write([new ClipboardItem({ "image/png": blob })]);
        showToast("コピーしました");
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

// --- Command mode (vim-style :) ---
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
    const cmd = raw.trim().toLowerCase();
    if (cmd === "tree") {
        showTree();
    }
}

cmdInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
        const val = cmdInput.value;
        hideCmd();
        execCmd(val);
    } else if (e.key === "Escape") {
        hideCmd();
    }
});

// --- Tree overlay ---
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
        row.innerHTML =
            `<span class="tr-idx">${fileIdx + 1}</span>` +
            `<span class="tr-time">${f.time || ""}</span>` +
            `<span class="tr-code">${f.code || ""}</span>` +
            `<span class="tr-name">${f.name || ""}</span>` +
            `<span class="tr-title">${f.title || ""}</span>`;
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

// --- Keyboard ---
let lastKey = "";
let lastKeyTime = 0;

document.addEventListener("keydown", (e) => {
    // Tree overlay key handling
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
            showGoto();
            break;
        case ":":
            e.preventDefault();
            showCmd();
            break;
        case "y":
            if (lastKey === "y" && now - lastKeyTime < 500) {
                copyCanvasToClipboard();
                lastKey = "";
                return;
            }
            break;
    }

    lastKey = e.key;
    lastKeyTime = now;
});

// --- Date controls ---
$("#prev-date").addEventListener("click", () => shiftDate(-1));
$("#next-date").addEventListener("click", () => shiftDate(1));
datePicker.addEventListener("change", () => {
    loadDate(datePicker.value.replace(/-/g, ""));
});

// --- Window resize ---
window.addEventListener("resize", () => {
    if (state.pdfDoc) renderPage();
});

// --- Init ---
(function init() {
    const today = new Date();
    datePicker.value = dateISO(today);
    loadDate(dateStr(today));
})();
