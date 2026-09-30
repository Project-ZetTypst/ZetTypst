// A stack is one reading context: following a link opens the target to the
// right of its source card, replacing whatever branch was there before. The
// URL records the stack (`?stack=a,b`), so history restores reading paths.
// Geometry and motion follow notes.andymatuschak.org.
import { entries, fetchCard, manifest, prefetch, routeURL, zkLink, type Entry } from './cards';

const stack = document.querySelector<HTMLElement>('.zk-stack')!;
const wide = matchMedia('(min-width: 801px)');
const panes = () => [...stack.querySelectorAll<HTMLElement>(':scope > .zk-pane')];
// A pane is obscured once less than a spine plus this margin remains visible.
const OBSCURE_MARGIN = 80;

const spineWidth = () => parseFloat(getComputedStyle(stack).getPropertyValue('--zk-spine')) || 40;

const titleOf = (pane: HTMLElement) => entries.get(pane.dataset.node ?? '')?.title ?? '';

function decorate(pane: HTMLElement, index: number, count: number) {
  pane.style.setProperty('--i', String(index));
  pane.style.setProperty('--n', String(count));
  const existing = pane.querySelector<HTMLElement>(':scope > .zk-spine');
  // Titles come from the manifest, which may still be loading.
  if (existing) { existing.textContent ||= titleOf(pane); return; }
  const spine = document.createElement('button');
  spine.className = 'zk-spine';
  spine.type = 'button';
  spine.tabIndex = -1;
  spine.textContent = titleOf(pane);
  spine.addEventListener('click', () => focus(pane));
  pane.prepend(spine);
}

/** Overlay: sliding over its predecessor. Obscured: reduced to about a spine. */
function layout() {
  const all = panes();
  all.forEach((pane, i) => decorate(pane, i, all.length));
  if (!wide.matches) return;
  const spine = spineWidth();
  const edge = stack.getBoundingClientRect().right;
  const lefts = all.map(pane => pane.getBoundingClientRect().left);
  all.forEach((pane, i) => {
    const visible = (lefts[i + 1] ?? edge) - lefts[i];
    const covering = i > 0 && lefts[i] < lefts[i - 1] + all[i - 1].offsetWidth - 1;
    const obscured = visible < spine + OBSCURE_MARGIN;
    pane.classList.toggle('is-obscured', obscured);
    pane.classList.toggle('is-overlay', covering || (obscured && i > 0));
  });
}

/** Scroll horizontally until the pane is fully visible, then to the card. */
function focus(pane: HTMLElement, id?: string) {
  const all = panes();
  const i = all.indexOf(pane);
  if (wide.matches && i >= 0) {
    const spine = spineWidth();
    const start = all.slice(0, i).reduce((sum, p) => sum + p.offsetWidth, 0);
    const lo = start + pane.offsetWidth + (all.length - 1 - i) * spine - stack.clientWidth;
    const hi = start - i * spine;
    const left = Math.max(Math.min(stack.scrollLeft, hi), Math.min(lo, hi));
    stack.scrollTo({ left: Math.max(0, left), behavior: 'smooth' });
  }
  // A document may hold several cards (a single-file export): bring this one up.
  const card = id ? cardIn(pane, id) : null;
  if (card && card !== pane.querySelector('[data-zk-node]')) {
    card.scrollIntoView({ block: 'start', behavior: 'smooth' });
  }
}

async function makePane(entry: Entry) {
  const pane = document.createElement('div');
  pane.className = 'zk-pane is-hidden';
  pane.dataset.node = entry.id;
  const article = document.createElement('article');
  try {
    article.append(await fetchCard(entry));
  } catch (error) {
    const link = document.createElement('a');
    link.href = routeURL(entry).href;
    link.textContent = `Open ${entry.title}`;
    article.append(link);
    console.error(error);
  }
  pane.append(article);
  return pane;
}

/** Enter as Andy's notes do: fade in while settling from 95% scale. */
function reveal(pane: HTMLElement) {
  requestAnimationFrame(() => requestAnimationFrame(() => pane.classList.remove('is-hidden')));
}

const cardIn = (pane: HTMLElement, id: string) =>
  pane.querySelector(`:scope > article > section[data-zk-node="${CSS.escape(id)}"]`);

/** The pane showing a card, if one does. */
const find = (entry: Entry) => panes().find(pane => cardIn(pane, entry.id));

/** Open `entry` to the right of `from`; an already open card is only focused. */
export async function open(entry: Entry, from?: HTMLElement) {
  const existing = find(entry);
  if (existing) return focus(existing, entry.id);
  const all = panes();
  const after = from ? all.indexOf(from) : all.length - 1;
  const pane = await makePane(entry);
  for (const stale of all.slice(after + 1)) stale.remove();
  stack.append(pane);
  layout();
  reveal(pane);
  record();
  focus(pane);
}

function record() {
  const ids = panes().slice(1).map(pane => pane.dataset.node).filter((id): id is string => !!id);
  const url = new URL(location.href);
  url.searchParams.delete('stack');
  // Commas stay literal: IDs are the readable part of a shared reading path.
  if (ids.length) url.search += (url.search ? '&' : '?') + 'stack=' + ids.map(encodeURIComponent).join(',');
  if (url.href !== location.href) history.pushState(null, '', url);
}

/** Rebuild the stack from the URL, keeping the panes that already match. */
async function restore() {
  await manifest;
  const ids = (new URL(location.href).searchParams.get('stack') ?? '').split(',').filter(Boolean);
  const wanted = ids.map(id => entries.get(id)).filter((it): it is Entry => !!it);
  const current = panes().slice(1);
  let keep = 0;
  while (keep < current.length && current[keep].dataset.node === wanted[keep]?.id) keep++;
  const added = await Promise.all(wanted.slice(keep).map(makePane));
  for (const pane of current.slice(keep)) pane.remove();
  stack.append(...added);
  layout();
  added.forEach(reveal);
  const last = panes().at(-1);
  if (last) focus(last);
}

export function initStack() {
  const home = panes()[0];
  home.dataset.node = home.querySelector('section[data-zk-node]')?.getAttribute('data-zk-node') ?? '';
  stack.addEventListener('click', event => {
    if (event.defaultPrevented || event.button !== 0 || !wide.matches
        || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
    const link = zkLink(event.target);
    if (!link) return;
    event.preventDefault();
    open(link.entry, link.element.closest<HTMLElement>('.zk-pane') ?? undefined);
  });
  // Hovering a link: warm its card, and light the spine of an open destination.
  stack.addEventListener('mouseover', event => {
    const link = zkLink(event.target);
    if (!link) return;
    prefetch(link.entry);
    find(link.entry)?.classList.add('is-destination');
  });
  stack.addEventListener('mouseout', event => {
    const link = zkLink(event.target);
    if (link) find(link.entry)?.classList.remove('is-destination');
  });
  stack.addEventListener('scroll', () => requestAnimationFrame(layout), { passive: true });
  addEventListener('resize', layout);
  wide.addEventListener('change', layout);
  addEventListener('popstate', restore);
  layout();
  manifest.then(layout);
  if (location.search.includes('stack=')) restore();
}
