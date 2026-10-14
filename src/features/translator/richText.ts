/** Only the restricted editor's inline nodes reach the Rust validator. */
export function plainFragment(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/\n/g, "<br/>");
}
export function hasContent(html: string): boolean {
  const doc = new DOMParser().parseFromString(html, "text/html");
  return (
    (doc.body.textContent ?? "").trim() !== "" ||
    doc.querySelector("img, svg, table") !== null
  );
}
export function editorContent(fragment: string): string {
  return `<p>${fragment}</p>`;
}
export function inlineFragment(html: string): string {
  const doc = new DOMParser().parseFromString(html, "text/html");
  const serialize = (node: Node): string => {
    if (node.nodeType === Node.TEXT_NODE) return plainFragment(node.textContent ?? "");
    if (!(node instanceof Element)) return "";
    const children = Array.from(node.childNodes).map(serialize).join("");
    if (node.tagName === "P") return children;
    const tag = node.tagName.toLowerCase();
    if (tag === "br") return "<br/>";
    if (tag === "img")
      return `<img src="${plainFragment(node.getAttribute("src") ?? "")}" alt="${plainFragment(node.getAttribute("alt") ?? "")}"/>`;
    if (["strong", "em", "u", "s"].includes(tag)) return `<${tag}>${children}</${tag}>`;
    if (tag === "a")
      return `<a href="${plainFragment(node.getAttribute("href") ?? "")}">${children}</a>`;
    return children;
  };
  return Array.from(doc.body.childNodes).map(serialize).join("<br/><br/>");
}

/**
 * Canonical document XHTML: retain the reader's block structure (including
 * original div/figure/section wrappers) and SVG cover artwork so the
 * rich-text preservation check can tell lossy conversions apart instead of
 * silently dropping them.
 */
const SVG_TAGS = [
  "svg",
  "g",
  "image",
  "path",
  "rect",
  "circle",
  "ellipse",
  "line",
  "polyline",
  "polygon",
  "title",
  "desc",
  "text",
  "tspan",
];
const SVG_ATTRS = [
  "href",
  "xlink:href",
  "viewBox",
  "preserveAspectRatio",
  "width",
  "height",
  "x",
  "y",
  "x1",
  "x2",
  "y1",
  "y2",
  "cx",
  "cy",
  "r",
  "rx",
  "ry",
  "d",
  "points",
  "transform",
  "opacity",
  "fill",
  "stroke",
  "stroke-width",
  "font-size",
  "text-anchor",
  "version",
];
export function documentFragment(html: string): string {
  const doc = new DOMParser().parseFromString(html, "text/html");
  const serialize = (node: Node): string => {
    if (node.nodeType === Node.TEXT_NODE) return plainFragment(node.textContent ?? "");
    if (!(node instanceof Element)) return "";
    const tag = node.tagName.toLowerCase();
    const allowed = [
      "p",
      "table",
      "thead",
      "tbody",
      "tfoot",
      "tr",
      "td",
      "th",
      "caption",
      "h1",
      "h2",
      "h3",
      "h4",
      "h5",
      "h6",
      "ul",
      "ol",
      "li",
      "blockquote",
      "div",
      "section",
      "article",
      "main",
      "header",
      "footer",
      "aside",
      "figure",
      "figcaption",
      "pre",
      "dl",
      "dt",
      "dd",
      "span",
      "b",
      "i",
      "code",
      "sup",
      "sub",
      "ruby",
      "rt",
      "rp",
      "strong",
      "em",
      "u",
      "s",
      "img",
      "br",
      "hr",
      "a",
      ...SVG_TAGS,
    ];
    const children = Array.from(node.childNodes).map(serialize).join("");
    if (!allowed.includes(tag)) return children;
    const names = [
      "class",
      "id",
      "style",
      "title",
      "lang",
      "dir",
      ...(tag === "img"
        ? ["src", "alt"]
        : tag === "a"
          ? ["href"]
          : tag === "td" || tag === "th"
            ? ["colspan", "rowspan"]
            : tag === "ol"
              ? ["start"]
              : []),
      ...(SVG_TAGS.includes(tag) ? SVG_ATTRS : []),
    ].sort();
    const attributes = names
      .filter((name) => node.hasAttribute(name))
      .map((name) => ` ${name}="${plainFragment(node.getAttribute(name) ?? "")}"`)
      .join("");
    if (
      [
        "br",
        "img",
        "hr",
        "image",
        "path",
        "rect",
        "circle",
        "ellipse",
        "line",
        "polyline",
        "polygon",
      ].includes(tag)
    )
      return `<${tag}${attributes}/>`;
    return `<${tag}${attributes}>${children}</${tag}>`;
  };
  return Array.from(doc.body.childNodes).map(serialize).join("");
}

/** Compare structural content without code indentation between block elements. */
export function comparableDocument(html: string): string {
  const document = new DOMParser().parseFromString(html, "text/html");
  for (const element of document.querySelectorAll("[style]")) {
    if (element instanceof HTMLElement)
      element.setAttribute("style", element.style.cssText);
  }
  for (const element of document.querySelectorAll(
    "body, ul, ol, blockquote, table, thead, tbody, tfoot, tr, div, section, article, main, header, footer, aside, figure, figcaption, pre, dl, svg, g",
  )) {
    for (const node of Array.from(element.childNodes)) {
      if (node.nodeType === Node.TEXT_NODE && !(node.textContent ?? "").trim())
        node.remove();
    }
  }
  return documentFragment(document.body.innerHTML);
}
