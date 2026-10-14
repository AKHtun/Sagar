/** Convert only sanitized image references to the platform's Tauri protocol URL. */
export function preparePreviewImages(
  html: string,
  convertResource: (path: string) => string,
): string {
  const document = new DOMParser().parseFromString(html, "text/html");
  for (const image of document.querySelectorAll("img[src], svg image")) {
    const attribute = image.localName === "img" ? "src" : "href";
    const source =
      image.getAttribute(attribute) ??
      image.getAttributeNS("http://www.w3.org/1999/xlink", "href") ??
      "";
    if (!source.startsWith("epubres://")) continue;
    const url = new URL(source);
    image.setAttribute(attribute, convertResource(`${url.hostname}${url.pathname}`));
    // Use SVG 2 href consistently; do not leave a second, stale image URL.
    if (attribute === "href") {
      image.removeAttributeNS("http://www.w3.org/1999/xlink", "href");
    }
  }
  return document.body.innerHTML;
}
