// Transient reading aids: hover previews and Forester-style in-place
// expansion of collapsed backmatter trees. Both act on note links only, as
// Typst marked them; a preview's links open in the stack, never nested.
import { entries, fetchView, manifest, zkLink } from './cards';
import { open } from './stack';

const hoverable = matchMedia('(hover: hover) and (pointer: fine)');

export function initPreview() {
  const popover = document.createElement('aside');
  popover.className = 'zk-preview';
  popover.hidden = true;
  document.body.append(popover);
  let timer = 0;
  let current: HTMLElement | null = null;
  // The pane holding the link whose preview is shown: its links open beside it.
  let origin: HTMLElement | undefined;
  // A clicked link was read, not glanced at: no preview until the pointer leaves it.
  let spent: Element | null = null;

  const hide = () => {
    clearTimeout(timer);
    timer = window.setTimeout(() => {
      popover.hidden = true;
      current = null;
    }, 180);
  };
  const place = (link: HTMLElement) => {
    const box = link.getBoundingClientRect();
    const width = Math.min(popover.offsetWidth, innerWidth - 16);
    popover.style.left = `${Math.max(8, Math.min(box.left, innerWidth - width - 8))}px`;
    const below = innerHeight - box.bottom > popover.offsetHeight + 12 || box.top < innerHeight / 2;
    popover.style.top = below ? `${box.bottom + 6}px` : '';
    popover.style.bottom = below ? '' : `${innerHeight - box.top + 6}px`;
  };

  document.addEventListener('mouseover', event => {
    if (!hoverable.matches) return;
    const target = event.target as Element;
    if (popover.contains(target)) { clearTimeout(timer); return; }
    // Links in the popover are not panes' links: previews do not nest.
    const found = target.closest('.zk-pane') ? zkLink(target) : undefined;
    if (!found) return;
    const { element: link, entry, source } = found;
    if (link === current || link === spent) return;
    // A link to another card previews it; a card's links to itself tell about it.
    const view = source === entry.id ? 'about' : 'preview';
    clearTimeout(timer);
    timer = window.setTimeout(async () => {
      current = link;
      try {
        const preview = await fetchView(entry, view);
        if (current !== link || !preview) return;
        origin = link.closest<HTMLElement>('.zk-pane') ?? undefined;
        popover.replaceChildren(preview);
        popover.hidden = false;
        popover.scrollTop = 0;
        place(link);
      } catch (error) {
        console.error(error);
      }
    }, 350);
  });
  document.addEventListener('mouseout', event => {
    const target = event.target as Element;
    const into = event.relatedTarget as Element | null;
    if (spent && target === spent && !(into && spent.contains(into))) spent = null;
    if ((target.closest('[data-zk-target]') || popover.contains(target)) && !(into && popover.contains(into))) hide();
  });
  // A note link in the preview continues the reading path from the source pane.
  popover.addEventListener('click', event => {
    const found = zkLink(event.target);
    if (!found || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
    event.preventDefault();
    open(found.entry, origin);
  });
  document.addEventListener('click', event => {
    clearTimeout(timer);
    spent = (event.target as Element).closest('[data-zk-target]');
    popover.hidden = true;
    current = null;
  }, true);
}

// A collapsed tree names its card in `data-zk-expand`.
export function initExpansion() {
  document.addEventListener('toggle', async event => {
    const details = event.target as HTMLDetailsElement;
    if (!details.open || !details.matches('details[data-zk-expand]') || details.dataset.loaded) return;
    details.dataset.loaded = 'loading';
    await manifest;
    const entry = entries.get(details.dataset.zkExpand ?? '');
    try {
      if (!entry) throw new Error(`Unknown card ${details.dataset.zkExpand}`);
      const view = await fetchView(entry, 'expand');
      const main = document.createElement('div');
      main.className = 'mainmatter';
      // This tree already has its summary, and a <details> holds only one:
      // a <details> view contributes its content without its own summary.
      for (const node of [...(view?.childNodes ?? [])]) {
        if (node instanceof HTMLDetailsElement) {
          main.append(...[...node.childNodes].filter(child => !(child instanceof HTMLElement && child.localName === 'summary')));
        } else {
          main.append(node);
        }
      }
      details.append(main);
      details.dataset.loaded = 'true';
    } catch (error) {
      delete details.dataset.loaded;
      console.error('Could not expand tree', error);
    }
  }, true);
}
