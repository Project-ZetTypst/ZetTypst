// ⌘K / Ctrl-K note search over the published titles.
import { manifest, routeURL, type Entry } from './cards';
import { open } from './stack';

const wide = matchMedia('(min-width: 800px)');

export function initPalette() {
  const dialog = document.querySelector<HTMLDialogElement>('.zk-palette')!;
  const input = dialog.querySelector('input')!;
  const list = dialog.querySelector('ul')!;
  let results: Entry[] = [];
  let selected = 0;

  const fold = (text: string) => text.normalize('NFKD').toLowerCase();
  const render = async () => {
    const all = await manifest;
    const terms = fold(input.value).split(/\s+/).filter(Boolean);
    results = all.filter(entry => {
      const text = fold(entry.title + ' ' + entry.id);
      return terms.every(term => text.includes(term));
    }).slice(0, 50);
    selected = Math.min(selected, Math.max(0, results.length - 1));
    list.replaceChildren(...results.map((entry, i) => {
      const item = document.createElement('li');
      item.setAttribute('role', 'option');
      item.setAttribute('aria-selected', String(i === selected));
      item.innerHTML = '<span class="title"></span><span class="id"></span>';
      item.querySelector('.title')!.textContent = entry.title;
      item.querySelector('.id')!.textContent = entry.id;
      item.addEventListener('mousemove', () => { if (selected !== i) { selected = i; mark(); } });
      item.addEventListener('click', () => choose(entry));
      return item;
    }));
  };
  const mark = () => [...list.children].forEach((item, i) => {
    item.setAttribute('aria-selected', String(i === selected));
    if (i === selected) item.scrollIntoView({ block: 'nearest' });
  });
  const choose = (entry: Entry) => {
    dialog.close();
    if (wide.matches) open(entry);
    else location.href = routeURL(entry).href;
  };
  const show = () => {
    input.value = '';
    selected = 0;
    render();
    dialog.showModal();
    input.focus();
  };

  input.addEventListener('input', () => { selected = 0; render(); });
  input.addEventListener('keydown', event => {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      const step = event.key === 'ArrowDown' ? 1 : -1;
      selected = (selected + step + results.length) % Math.max(1, results.length);
      mark();
    } else if (event.key === 'Enter' && results[selected]) {
      event.preventDefault();
      choose(results[selected]);
    }
  });
  dialog.addEventListener('click', event => { if (event.target === dialog) dialog.close(); });
  document.querySelector('.zk-search')?.addEventListener('click', show);
  document.addEventListener('keydown', event => {
    const typing = (event.target as Element).closest('input, textarea, [contenteditable]');
    if ((event.key === 'k' && (event.metaKey || event.ctrlKey)) || (event.key === '/' && !typing)) {
      event.preventDefault();
      dialog.open ? dialog.close() : show();
    }
  });
}
