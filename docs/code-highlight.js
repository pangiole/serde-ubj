'use strict';

// Fragment highlighting for mdBook code blocks.
//
// Wrap a fragment of a fenced code block in the pair of Rust block comments
//
//     /*<highlight>*/ ... /*</highlight>*/
//
// Both are ordinary block comments, so the code keeps compiling (doctests,
// `mdbook test`, the playground, ...). After highlight.js has run, this script
// removes the two delimiter comments and wraps the enclosed fragment in a
// `<span class="code-hl">`, which docs/code-highlight.css paints.
//
// Delimiters are only recognized when highlight.js classified them as a
// `hljs-comment`, which keeps a `/*<highlight>*/` inside a string literal from
// being treated as a marker. Pairs are matched left to right; nesting and
// overlapping pairs are not supported.

(function codeHighlight() {
    const START = '/*<highlight>*/';
    const END = '/*</highlight>*/';
    const CLASS = 'code-hl';

    // Collect the text nodes of `root` together with their cumulative
    // character offsets, and return the concatenated text.
    function textNodes(root) {
        const segments = [];
        const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
        let node;
        let text = '';
        while ((node = walker.nextNode()) !== null) {
            segments.push({
                node,
                start: text.length,
                end: text.length + node.nodeValue.length,
            });
            text += node.nodeValue;
        }
        return { segments, text };
    }

    // The segment containing character `offset` (start inclusive, end
    // exclusive).
    function segmentAt(segments, offset) {
        for (const segment of segments) {
            if (offset >= segment.start && offset < segment.end) {
                return segment;
            }
        }
        return null;
    }

    // Whether the character at `offset` sits inside a highlighted block
    // comment.
    function insideComment(segments, offset) {
        const segment = segmentAt(segments, offset);
        if (segment === null) {
            return false;
        }
        const element = segment.node.parentElement;
        return element !== null && element.closest('.hljs-comment') !== null;
    }

    // Map a global character offset to a DOM (node, offset) position.
    function positionAt(segments, offset) {
        for (const segment of segments) {
            if (offset <= segment.end) {
                return { node: segment.node, offset: offset - segment.start };
            }
        }
        const last = segments[segments.length - 1];
        return { node: last.node, offset: last.node.nodeValue.length };
    }

    function findAll(text, needle) {
        const offsets = [];
        let index = 0;
        while ((index = text.indexOf(needle, index)) !== -1) {
            offsets.push(index);
            index += needle.length;
        }
        return offsets;
    }

    // Sequential (non nested) pairs of comment-delimited markers.
    function markerPairs(text, segments) {
        const starts = findAll(text, START).filter((offset) => insideComment(segments, offset));
        const ends = findAll(text, END).filter((offset) => insideComment(segments, offset));

        const pairs = [];
        let s = 0;
        let e = 0;
        while (s < starts.length && e < ends.length) {
            if (ends[e] <= starts[s]) {
                e += 1;
                continue;
            }
            pairs.push({ start: starts[s], end: ends[e] });
            s += 1;
            e += 1;
        }
        return pairs;
    }

    // Delete `count` characters from the start of `container`.
    function deleteFromStart(container, count) {
        let remaining = count;
        while (remaining > 0 && container.firstChild !== null) {
            const node = container.firstChild;
            if (node.nodeType === Node.TEXT_NODE) {
                const take = Math.min(remaining, node.nodeValue.length);
                node.nodeValue = node.nodeValue.slice(take);
                remaining -= take;
                if (node.nodeValue.length === 0) {
                    container.removeChild(node);
                }
            } else {
                const length = node.textContent.length;
                if (length <= remaining) {
                    container.removeChild(node);
                    remaining -= length;
                } else {
                    deleteFromStart(node, remaining);
                    remaining = 0;
                }
            }
        }
    }

    // Delete `count` characters from the end of `container`.
    function deleteFromEnd(container, count) {
        let remaining = count;
        while (remaining > 0 && container.lastChild !== null) {
            const node = container.lastChild;
            if (node.nodeType === Node.TEXT_NODE) {
                const keep = Math.max(0, node.nodeValue.length - remaining);
                const removed = node.nodeValue.length - keep;
                node.nodeValue = node.nodeValue.slice(0, keep);
                remaining -= removed;
                if (node.nodeValue.length === 0) {
                    container.removeChild(node);
                }
            } else {
                const length = node.textContent.length;
                if (length <= remaining) {
                    container.removeChild(node);
                    remaining -= length;
                } else {
                    deleteFromEnd(node, remaining);
                    remaining = 0;
                }
            }
        }
    }

    // Wrap one [START ... END] pair in a `.code-hl` span.
    function wrapPair(segments, pair) {
        const start = positionAt(segments, pair.start);
        const end = positionAt(segments, pair.end + END.length);

        const range = document.createRange();
        range.setStart(start.node, start.offset);
        range.setEnd(end.node, end.offset);

        const wrapper = document.createElement('span');
        wrapper.className = CLASS;
        wrapper.appendChild(range.extractContents());
        deleteFromStart(wrapper, START.length);
        deleteFromEnd(wrapper, END.length);
        range.insertNode(wrapper);
    }

    function processBlock(code) {
        if (code.querySelector('.' + CLASS) !== null) {
            return;
        }
        const { segments, text } = textNodes(code);
        if (text.length === 0) {
            return;
        }
        const pairs = markerPairs(text, segments);
        // Right to left, so the offsets of the earlier pairs stay valid after
        // the DOM has been split by the preceding extraction.
        for (let i = pairs.length - 1; i >= 0; i -= 1) {
            wrapPair(segments, pairs[i]);
        }
        // The extraction leaves the (now empty) delimiter comment spans
        // behind; drop them so the rendered markup stays tidy.
        if (pairs.length > 0) {
            code.querySelectorAll('.hljs-comment').forEach((element) => {
                if (element.textContent.length === 0) {
                    element.remove();
                }
            });
        }
    }

    function run() {
        document.querySelectorAll('pre code').forEach(processBlock);
    }

    // additional-js runs after book.js, which has already highlighted the code
    // blocks synchronously. The extra listeners are idempotent safety nets.
    run();
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', run);
    }
    window.addEventListener('load', run);
})();
