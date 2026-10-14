import { Node } from "@tiptap/react";

/** Empty EPUB navigation targets are nodes, not links attached to text. */
export const BookAnchor = Node.create({
  name: "bookAnchor",
  group: "inline",
  inline: true,
  atom: true,
  selectable: false,
  parseHTML() {
    return [
      {
        tag: "a[id]",
        priority: 100,
        getAttrs: (element) => (element.childNodes.length === 0 ? {} : false),
      },
    ];
  },
  renderHTML({ HTMLAttributes }) {
    return ["a", HTMLAttributes];
  },
});
