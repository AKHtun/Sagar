import Image from "@tiptap/extension-image";
import { Node } from "@tiptap/react";
import { documentFragment } from "./richText";

/** Structural XHTML containers remain in the document while their text is edited. */
const containers = [
  "div",
  "section",
  "article",
  "main",
  "header",
  "footer",
  "aside",
  "figure",
  "figcaption",
  "dl",
  "dd",
];
const blockSelector =
  "p,h1,h2,h3,h4,h5,h6,ul,ol,li,blockquote,hr,table,div,section,article,main,header,footer,aside,figure,figcaption,pre,dl,dt,dd,svg,picture,img";
const inline = ["span", "b", "i", "sup", "sub", "ruby", "rt", "rp"];

export const BookStructure = [
  Image.extend({
    name: "book_inline_image",
    parseHTML: () => [
      {
        tag: "img[src]",
        priority: 120,
        getAttrs: (element) =>
          element.parentElement?.closest(
            "p,h1,h2,h3,h4,h5,h6,span,a,pre,dt,sup,sub,ruby,rt,rp",
          )
            ? {}
            : false,
      },
    ],
  }).configure({ inline: true, allowBase64: true }),
  ...["svg", "picture"].map((tag) =>
    Node.create({
      name: `book_${tag}`,
      group: "block",
      atom: true,
      addAttributes: () => ({
        original: { default: "", rendered: false },
      }),
      parseHTML: () => [
        {
          tag,
          priority: 110,
          getAttrs: (element) => ({ original: documentFragment(element.outerHTML) }),
        },
      ],
      renderHTML: ({ node }) => {
        const holder = document.createElement("div");
        holder.innerHTML = node.attrs.original as string;
        return holder.firstElementChild ?? document.createElement("div");
      },
      // Never execute or fetch embedded artwork in the privileged editor DOM.
      // Serialization retains its markup; the Rust validator owns export safety.
      addNodeView: () => () => {
        const dom = document.createElement("div");
        dom.className = "book-protected-content";
        dom.contentEditable = "false";
        dom.textContent =
          tag === "svg"
            ? "SVG artwork — preserved. Edit in HTML view."
            : "Responsive image — preserved. Edit in HTML view.";
        return { dom };
      },
    }),
  ),
  ...containers.map((tag) =>
    Node.create({
      name: `book_${tag}`,
      group: "block",
      content: "block*",
      defining: true,
      parseHTML: () => [
        {
          tag,
          priority: 110,
          getAttrs: (element) =>
            Array.from(element.children).some((child) => child.matches(blockSelector))
              ? {}
              : false,
        },
      ],
      renderHTML: ({ HTMLAttributes }) => [tag, HTMLAttributes, 0],
    }),
  ),
  ...containers.map((tag) =>
    Node.create({
      name: `book_text_${tag}`,
      group: "block",
      content: "inline*",
      defining: true,
      parseHTML: () => [
        {
          tag,
          priority: 111,
          getAttrs: (element) =>
            Array.from(element.children).some((child) => child.matches(blockSelector))
              ? false
              : {},
        },
      ],
      renderHTML: ({ HTMLAttributes }) => [tag, HTMLAttributes, 0],
    }),
  ),
  ...["pre", "dt"].map((tag) =>
    Node.create({
      name: `book_${tag}`,
      group: "block",
      content: "inline*",
      whitespace: tag === "pre" ? "pre" : "normal",
      defining: true,
      parseHTML: () => [
        {
          tag,
          priority: 110,
          getAttrs: (element) =>
            tag === "pre" &&
            element.childNodes.length === 1 &&
            element.firstElementChild?.tagName === "CODE"
              ? false
              : {},
        },
      ],
      renderHTML: ({ HTMLAttributes }) => [tag, HTMLAttributes, 0],
    }),
  ),
  ...inline.map((tag) =>
    Node.create({
      name: `book_${tag}`,
      group: "inline",
      inline: true,
      content: "inline*",
      parseHTML: () => [{ tag, priority: 110 }],
      renderHTML: ({ HTMLAttributes }) => [tag, HTMLAttributes, 0],
    }),
  ),
];

export const bookStructureNames = BookStructure.map((extension) => extension.name);
