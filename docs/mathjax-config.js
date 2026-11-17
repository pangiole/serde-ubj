// Configure MathJax v3 (tex-svg bundle) for the mdBook.
//
// This file MUST be loaded before `tex-svg.js` (see `additional-js` order in
// `book.toml`): MathJax reads `window.MathJax` when the bundle is evaluated.
//
// It enables the usual LaTeX delimiters, including the `$ ... $` inline form
// that the book already uses (e.g. in 03-data-models.md).
window.MathJax = {
    tex: {
        inlineMath: [['$', '$'], ['\\(', '\\)']],
        displayMath: [['$$', '$$'], ['\\[', '\\]']],
        processEscapes: true,
    },
    svg: {
        // Share the glyph cache across the page to keep the DOM smaller.
        fontCache: 'global',
    },
    options: {
        // Skip elements that are not regular content (mirrors mdBook's own
        // escaping hooks and keeps MathJax out of code blocks' wrappers).
        ignoreHtmlClass: 'tex2jax_ignore',
        processHtmlClass: 'tex2jax_process',
    },
};
