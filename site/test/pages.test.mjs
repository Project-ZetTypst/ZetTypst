import assert from 'node:assert/strict';
import { afterEach, beforeEach, test } from 'node:test';

let pool, idle, requests, parsed;
let sequence = 0;
const original = { window: globalThis.window, fetch: globalThis.fetch, DOMParser: globalThis.DOMParser };
const url = name => new URL(name, 'https://notes.example/');
const tick = () => new Promise(resolve => setImmediate(resolve));
const runIdle = () => idle.splice(0).forEach(work => work());
const respond = (request, status = 200) => request.resolve({
  ok: status === 200, status, text: async () => request.path,
});

beforeEach(async () => {
  idle = []; requests = []; parsed = [];
  globalThis.window = { requestIdleCallback: work => idle.push(work), setTimeout: work => idle.push(work) };
  globalThis.fetch = (path, options) => new Promise((resolve, reject) => requests.push({ path, options, resolve, reject }));
  globalThis.DOMParser = class {
    parseFromString(text, type) {
      parsed.push(text);
      return { text, type };
    }
  };
  // Each test gets an independent pool without exposing reset hooks in production.
  pool = await import(`../src/client/pages.mjs?test=${sequence++}`);
});
afterEach(() => Object.assign(globalThis, original));

test('background downloads start at idle, with at most two in flight', async () => {
  for (const name of ['a', 'b', 'c']) pool.loadPage(url(name), true);
  assert.equal(requests.length, 0);
  runIdle();
  assert.deepEqual(requests.map(it => it.path), ['/a', '/b']);
  assert.ok(requests.every(it => it.options.priority === 'low'));
  respond(requests[0]);
  await tick();
  runIdle();
  assert.deepEqual(requests.map(it => it.path), ['/a', '/b', '/c']);
});

test('cards sharing a canonical page share its pending and completed result', async () => {
  const first = pool.loadPage(url('a/index.html#one'), true);
  assert.equal(pool.loadPage(url('a/index.html#two'), true), first);
  runIdle(); respond(requests[0]); await tick(); runIdle();
  const page = await first;
  assert.equal(await pool.loadPage(url('a/index.html#three')), page);
  assert.equal(requests.length, 1);
  assert.deepEqual(parsed, ['/a/index.html']);
});

test('opening a queued destination bypasses occupied background slots', async () => {
  for (const name of ['a', 'b', 'c']) pool.loadPage(url(name), true);
  runIdle();
  const target = pool.loadPage(url('c'));
  assert.deepEqual(requests.map(it => it.path), ['/a', '/b', '/c']);
  assert.equal(requests[2].options.priority, 'high');
  respond(requests[2]);
  assert.equal((await target).text, '/c');
  runIdle();
  assert.equal(requests.length, 3);
});

test('hover and clicking an active prefetch share its request', async () => {
  const prepared = pool.loadPage(url('a'), true);
  runIdle();
  const hover = pool.loadPage(url('a'));
  const click = pool.loadPage(url('a'));
  assert.equal(prepared, hover);
  assert.equal(hover, click);
  assert.equal(requests.length, 1);
  respond(requests[0]);
  assert.equal((await click).text, '/a');
  assert.deepEqual(parsed, ['/a']);
});

test('background parsing waits for idle but an interactive request promotes it', async () => {
  const prepared = pool.loadPage(url('a'), true);
  runIdle(); respond(requests[0]); await tick();
  assert.deepEqual(parsed, []);
  assert.equal(pool.loadPage(url('a')), prepared);
  assert.equal((await prepared).text, '/a');
  runIdle();
  assert.deepEqual(parsed, ['/a']);
  assert.equal(requests.length, 1);
});

test('queued background work pauses while an interactive page is loading', async () => {
  pool.loadPage(url('a'), true);
  const opened = pool.loadPage(url('b'));
  runIdle();
  assert.deepEqual(requests.map(it => it.path), ['/b']);
  respond(requests[0]); await opened; await tick(); runIdle();
  assert.deepEqual(requests.map(it => it.path), ['/b', '/a']);
});

test('HTTP and network failures release their entries and permit retries', async () => {
  for (const failure of ['http', 'network']) {
    const first = pool.loadPage(url(failure));
    const rejected = assert.rejects(first);
    if (failure === 'http') respond(requests.at(-1), 401);
    else requests.at(-1).reject(new Error('offline'));
    await rejected;
    const second = pool.loadPage(url(failure));
    assert.notEqual(second, first);
    respond(requests.at(-1));
    assert.equal((await second).text, '/' + failure);
  }
});

test('a failed background request frees its slot for the next neighbour', async () => {
  for (const name of ['a', 'b', 'c']) pool.loadPage(url(name), true).catch(() => {});
  runIdle(); requests[0].reject(new Error('offline'));
  await tick(); runIdle();
  assert.deepEqual(requests.map(it => it.path), ['/a', '/b', '/c']);
});

test('a parse failure releases the foreground queue and can be retried', async () => {
  pool.loadPage(url('a'), true);
  const parser = globalThis.DOMParser;
  globalThis.DOMParser = class { parseFromString() { throw new Error('parse failed'); } };
  const opened = pool.loadPage(url('b'));
  const rejected = assert.rejects(opened, /parse failed/);
  respond(requests[0]); await rejected;
  globalThis.DOMParser = parser;
  await tick(); runIdle();
  assert.deepEqual(requests.map(it => it.path), ['/b', '/a']);
  const retried = pool.loadPage(url('b'));
  respond(requests.at(-1));
  assert.equal((await retried).text, '/b');
});

test('the initial canonical page is reused without downloading or parsing', async () => {
  const current = { text: 'already displayed' };
  pool.rememberPage(url('index.html'), current);
  assert.equal(await pool.loadPage(url('index.html#self'), true), current);
  assert.equal(await pool.loadPage(url('index.html')), current);
  runIdle();
  assert.equal(requests.length, 0);
  assert.deepEqual(parsed, []);
});

test('browsers without idle callbacks yield through a timer', async () => {
  delete window.requestIdleCallback;
  const prepared = pool.loadPage(url('a'), true);
  assert.equal(requests.length, 0);
  runIdle(); respond(requests[0]); await tick(); runIdle();
  assert.equal((await prepared).text, '/a');
});
