// Build-time math typesetting: Typst's MathML becomes MathJax CHTML, with the
// MathML kept for assistive technology. Pages need no math runtime, and a card
// moved into another page (stack, preview, expansion) renders unchanged
// because every page shares one stylesheet covering all typeset characters.
// Typst's MathML is first restated in standard MathML (./standardize.mjs).
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { mathjax } from '@mathjax/src/js/mathjax.js';
import { MathML } from '@mathjax/src/js/input/mathml.js';
import { CHTML } from '@mathjax/src/js/output/chtml.js';
import { liteAdaptor } from '@mathjax/src/js/adaptors/liteAdaptor.js';
import { RegisterHTMLHandler } from '@mathjax/src/js/handlers/html.js';
import { AssistiveMmlHandler } from '@mathjax/src/js/a11y/assistive-mml.js';
import '@mathjax/src/js/util/asyncLoad/esm.js';
import { parse } from 'node-html-parser';
import { standardize } from './standardize.mjs';

const require = createRequire(import.meta.url);

export const fontDirectory = join(
  dirname(require.resolve('@mathjax/mathjax-newcm-font/package.json')), 'chtml', 'woff2');
export const licenseFile = join(
  dirname(require.resolve('@mathjax/src/package.json')), 'LICENSE');

/**
 * @param {object} options
 * @param {string} options.fontURL Font directory, relative to the stylesheet.
 */
export function createTypesetter({ fontURL }) {
  const adaptor = liteAdaptor();
  AssistiveMmlHandler(RegisterHTMLHandler(adaptor));
  const input = new MathML();
  // The page's CSS, not a guess at the reader's fonts, sets the math size.
  const output = new CHTML({ fontURL, displayOverflow: 'scroll', matchFontHeight: false });
  const fail = (_document, _math, error) => { throw error; };
  // Page HTML → typeset HTML. Unchanged pages are served from here, and their
  // characters remain in the shared stylesheet.
  const cache = new Map();
  let queue = Promise.resolve();

  // Only the <math> elements go through MathJax; everything else in the page,
  // SVG frames included, is kept byte for byte.
  async function render(html) {
    const ranges = parse(html).querySelectorAll('math').map(math => math.range);
    const slots = ranges.map(([start, end], i) =>
      `<mjx-slot id="s${i}">${standardize(html.slice(start, end))}</mjx-slot>`);
    const document = mathjax.document(`<body>${slots.join('')}</body>`, {
      InputJax: input,
      OutputJax: output,
      compileError: fail,
      typesetError: fail,
    });
    await document.renderPromise();
    const typeset = ranges.map((_, i) => adaptor.innerHTML(adaptor.getElement(`#s${i}`, document.document)));
    let result = '';
    let cursor = 0;
    ranges.forEach(([start, end], i) => {
      result += html.slice(cursor, start) + typeset[i];
      cursor = end;
    });
    return result + html.slice(cursor);
  }

  return {
    /** Typeset one page; calls are serialized because the output is shared. */
    typeset(html) {
      if (!html.includes('<math')) return Promise.resolve(html);
      const key = createHash('sha256').update(html).digest('hex');
      if (!cache.has(key)) {
        const result = queue.then(() => render(html));
        queue = result.catch(() => {});
        cache.set(key, result);
        result.catch(() => cache.delete(key));
      }
      return cache.get(key);
    },
    /** CSS for every character typeset so far, and the font files it names. */
    async stylesheet() {
      await queue;
      const sheet = output.chtmlStyles;
      const css = sheet ? adaptor.textContent(sheet) : '';
      const fonts = [...new Set([...css.matchAll(/url\("?([^")]+?\.woff2)"?\)/g)]
        .map(match => match[1].slice(fontURL.length + 1)))];
      return { css, fonts };
    },
  };
}
