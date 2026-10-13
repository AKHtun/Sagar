import { Extension } from "@tiptap/react";

/** Retain source XHTML attributes when switching/editing in rich text. */
export const BookAttributes = Extension.create({
  name: "bookAttributes",
  addGlobalAttributes() {
    return [
      {
        types: [
          "paragraph",
          "table",
          "tableRow",
          "tableCell",
          "tableHeader",
          "heading",
          "bulletList",
          "orderedList",
          "listItem",
          "blockquote",
          "horizontalRule",
          "image",
          "link",
          "bold",
          "italic",
          "underline",
          "strike",
        ],
        attributes: Object.fromEntries(
          ["class", "id", "style", "title", "lang", "dir"].map((name) => [
            name,
            {
              default: null,
              parseHTML: (element: HTMLElement) => element.getAttribute(name),
              renderHTML: (attributes: Record<string, unknown>) =>
                typeof attributes[name] === "string" ? { [name]: attributes[name] } : {},
            },
          ]),
        ),
      },
    ];
  },
});
