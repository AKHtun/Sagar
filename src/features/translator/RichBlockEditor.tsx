import Image from "@tiptap/extension-image";
import { type Editor, EditorContent, useEditor } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { useEffect } from "react";
import { editorContent, inlineFragment } from "./richText";

export function RichBlockEditor({
  html,
  disabled,
  label,
  onChange,
  onFocus,
}: {
  html: string;
  disabled: boolean;
  label: string;
  onChange: (html: string) => void;
  onFocus: (editor: Editor) => void;
}) {
  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        heading: false,
        blockquote: false,
        bulletList: false,
        orderedList: false,
        listItem: false,
        listKeymap: false,
        code: false,
        codeBlock: false,
        horizontalRule: false,
        link: { openOnClick: false, autolink: false },
      }),
      Image.configure({ inline: true, allowBase64: true }),
    ],
    content: editorContent(html),
    editable: !disabled,
    editorProps: {
      attributes: {
        class: "rich-block",
        role: "textbox",
        "aria-multiline": "true",
        "aria-label": label,
        "data-placeholder": "Write your translation…",
      },
    },
    onUpdate: ({ editor }) => onChange(inlineFragment(editor.getHTML())),
    onFocus: ({ editor }) => onFocus(editor),
  });
  useEffect(() => {
    if (editor && inlineFragment(editor.getHTML()) !== html)
      editor.commands.setContent(editorContent(html), { emitUpdate: false });
  }, [editor, html]);
  useEffect(() => {
    editor?.setEditable(!disabled, false);
  }, [editor, disabled]);
  return <EditorContent editor={editor} />;
}
