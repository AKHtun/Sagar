import { TextSelection } from "@tiptap/pm/state";
import type { Editor } from "@tiptap/react";

/** Keep the book schema's inline containers rather than reparse them as marks.
 * In particular, footnote IDs and nesting must survive opening the editor.
 * Complex ranges stay disabled rather than flattening their XHTML structure.
 */
export function toggleBookInline(editor: Editor, name: string, apply = true) {
  const { state } = editor;
  const { selection } = state;
  const { $from, $to, from, to } = selection;
  const type = state.schema.nodes[name];
  if (
    !editor.isEditable ||
    !(selection instanceof TextSelection) ||
    selection.empty ||
    !$from.sameParent($to) ||
    !type ||
    !($from.parent.isTextblock || $from.parent.isInline)
  )
    return false;
  const parent = $from.parent;
  if (parent.type === type) {
    // Do not discard semantic IDs/attributes or change unselected text.
    if (
      from !== $from.start() ||
      to !== $to.end() ||
      Object.values(parent.attrs).some(
        (value) =>
          value !== null && (typeof value !== "object" || Object.keys(value).length > 0),
      )
    )
      return false;
    if (apply) {
      const tr = state.tr.replaceWith($from.before(), $to.after(), parent.content);
      tr.setSelection(TextSelection.create(tr.doc, from - 1, to - 1));
      editor.view.dispatch(tr.scrollIntoView());
    }
    return true;
  }
  const content = state.doc.slice(from, to).content;
  if (!type.validContent(content)) return false;
  if (apply) {
    const tr = state.tr.replaceWith(from, to, type.create(null, content));
    tr.setSelection(TextSelection.create(tr.doc, from + 1, to + 1));
    editor.view.dispatch(tr.scrollIntoView());
  }
  return true;
}
