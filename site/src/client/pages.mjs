// One in-memory page pool for background preparation and interactive reading.
// Only displayed cards enqueue neighbours; downloaded documents do not.
/** @typedef {{ promise: Promise<Document>, start: () => void, promote: () => void }} Page */
/** @type {Map<string, Page>} */
const pages = new Map();
/** @type {Set<Page>} */
const waiting = new Set();
let background = 0;
let foreground = 0;
let scheduled = false;

/** @param {() => void} work */
function idle(work) {
  if (typeof window.requestIdleCallback === 'function') window.requestIdleCallback(work);
  else window.setTimeout(work, 0);
}

function pump() {
  if (scheduled || !waiting.size) return;
  scheduled = true;
  idle(() => {
    scheduled = false;
    if (foreground) return;
    for (const page of waiting) {
      if (background >= 2) break;
      page.start();
    }
  });
}

/** @param {string} key @returns {Page} */
function create(key) {
  /** @type {(document: Document) => void} */
  let resolve;
  /** @type {(error: unknown) => void} */
  let reject;
  /** @type {Promise<Document>} */
  const promise = new Promise((done, fail) => { resolve = done; reject = fail; });
  let urgent = false;
  let started = false;
  let finished = false;
  /** @param {unknown} [error] */
  const finish = (error = undefined) => {
    finished = true;
    if (urgent) foreground--;
    if (error !== undefined) { pages.delete(key); reject(error); }
    pump();
  };
  // Once downloaded, promotion runs the pending parse rather than fetching again.
  let resume = () => {
    if (started) return;
    started = true;
    waiting.delete(page);
    const low = !urgent;
    if (low) background++;
    fetch(key, { priority: low ? 'low' : 'high' })
      .then(response => {
        if (!response.ok) throw new Error(`HTTP ${response.status}: ${key}`);
        return response.text();
      })
      .then(text => {
        let parsed = false;
        resume = () => {
          if (parsed) return;
          parsed = true;
          try {
            resolve(new DOMParser().parseFromString(text, 'text/html'));
            finish();
          } catch (error) { finish(error); }
        };
        if (urgent) resume();
        else idle(() => resume());
      })
      .catch(error => finish(error))
      .finally(() => { if (low) background--; pump(); });
  };
  const page = {
    promise,
    start: () => resume(),
    promote: () => {
      if (finished) return;
      if (!urgent) { urgent = true; foreground++; }
      resume();
    },
  };
  return page;
}

/**
 * Interactive requests bypass the background queue and share its result.
 * @param {URL} url @param {boolean} preload @returns {Promise<Document>}
 */
export function loadPage(url, preload = false) {
  const key = url.pathname;
  let page = pages.get(key);
  if (!page) {
    page = create(key);
    pages.set(key, page);
    if (preload) { waiting.add(page); pump(); }
  }
  if (!preload) page.promote();
  return page.promise;
}

/**
 * The canonical page already on screen needs no second download or parse.
 * @param {URL} url @param {Document} document
 */
export function rememberPage(url, document) {
  pages.set(url.pathname, { promise: Promise.resolve(document), start() {}, promote() {} });
}
