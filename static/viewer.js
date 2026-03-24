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
    date: "",
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
async function loadPdf(filename) {
    loading.style.display = "block";
    canvas.style.display = "none";

    if (state.pdfDoc) {
        state.pdfDoc.destroy();
        state.pdfDoc = null;
    }

    const url = `/api/pdf/${filename}`;
    const doc = await pdfjsLib.getDocument(url).promise;
    state.pdfDoc = doc;
    state.pageCount = doc.numPages;
    state.pageNum = 1;

    loading.style.display = "none";
    canvas.style.display = "block";

    await renderPage();
}

async function renderPage() {
    if (!state.pdfDoc || state.rendering) return;
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
    state.date = date;
    state.files = [];
    state.fileIdx = 0;
    canvas.style.display = "none";
    emptyMsg.style.display = "none";
    loading.style.display = "block";

    const items = await fetchList(date);
    state.files = items;

    if (items.length === 0) {
        loading.style.display = "none";
        emptyMsg.style.display = "block";
        updateUI();
        return;
    }

    await loadPdf(items[0].pdf);
}

// --- Keyboard ---
document.addEventListener("keydown", (e) => {
    if (e.target.tagName === "INPUT") return;

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
    }
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
