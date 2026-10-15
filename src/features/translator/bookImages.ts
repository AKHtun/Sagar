import { convertFileSrc } from "@tauri-apps/api/core";
import type { NodeViewRenderer } from "@tiptap/react";

/** Display-only URL resolution. Saved XHTML keeps its original relative src. */
export function bookImageUrl(src: string, base: string): string | null {
  if (/^data:image\/(png|jpeg|gif|webp|avif|bmp);base64,[A-Za-z0-9+/=\s]+$/.test(src))
    return src;
  if (
    !base.startsWith("epubres://") ||
    !src ||
    src.startsWith("/") ||
    src.includes(":") ||
    src.includes("\\")
  )
    return null;
  try {
    const url = new URL(src, base);
    return url.protocol === "epubres:" && url.host === new URL(base).host
      ? convertFileSrc(`${url.hostname}${url.pathname}`, "epubres")
      : null;
  } catch {
    return null;
  }
}

export function bookImageView(base: string): NodeViewRenderer {
  return ({ node }) => {
    const dom = document.createElement("img");
    const update = (next: typeof node) => {
      if (next.type !== node.type) return false;
      const src = bookImageUrl(String(next.attrs.src ?? ""), base);
      if (src) dom.src = src;
      else dom.removeAttribute("src");
      dom.alt = String(next.attrs.alt ?? "Book image");
      for (const name of ["width", "height"]) {
        const value = String(next.attrs[name] ?? "");
        if (/^\d+(?:\.\d+)?%?$/.test(value)) dom.setAttribute(name, value);
        else dom.removeAttribute(name);
      }
      dom.title = String(next.attrs.title ?? "");
      return true;
    };
    update(node);
    return { dom, update };
  };
}
