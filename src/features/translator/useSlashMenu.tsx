import type { Editor } from "@tiptap/react";
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Icon, type IconName } from "../../components/Icon";

type CommandId =
  | "text"
  | "h1"
  | "h2"
  | "h3"
  | "table"
  | "image"
  | "blockquote"
  | "code"
  | "inlineCode";
const commands: {
  id: CommandId;
  label: string;
  detail: string;
  keywords: string;
  icon?: IconName;
}[] = [
  {
    id: "text",
    label: "Text",
    detail: "Plain paragraph",
    keywords: "paragraph normal",
    icon: "edit",
  },
  { id: "h1", label: "H1", detail: "Large heading", keywords: "heading 1 title" },
  { id: "h2", label: "H2", detail: "Medium heading", keywords: "heading 2" },
  { id: "h3", label: "H3", detail: "Small heading", keywords: "heading 3" },
  {
    id: "blockquote",
    label: "Blockquote",
    detail: "Quote a passage",
    keywords: "quote quotation",
    icon: "blockquote",
  },
  {
    id: "code",
    label: "Code block",
    detail: "Preformatted code with preserved line breaks",
    keywords: "pre monospace codeblock",
    icon: "codeBlock",
  },
  {
    id: "inlineCode",
    label: "Inline code",
    detail: "Format the next text as code",
    keywords: "inline code monospace",
    icon: "code",
  },
  {
    id: "table",
    label: "Table",
    detail: "Insert a 3 × 3 table",
    keywords: "grid cells",
    icon: "table",
  },
  {
    id: "image",
    label: "Image",
    detail: "Import or reuse a chapter image",
    keywords: "picture photo",
    icon: "image",
  },
];

/** One transaction removes the slash query and applies formatting, including undo. */
export function runSlashCommand(
  editor: Editor,
  id: CommandId,
  range: { from: number; to: number },
  onImage: () => void,
) {
  if (!editor.isEditable || editor.isDestroyed) return false;
  const chain = editor.chain().focus().deleteRange(range);
  if (id === "text") return chain.setParagraph().run();
  if (id === "h1" || id === "h2" || id === "h3")
    return chain.setHeading({ level: Number(id[1]) as 1 | 2 | 3 }).run();
  if (id === "blockquote") return chain.toggleBlockquote().run();
  if (id === "code") return chain.setCodeBlock().run();
  if (id === "inlineCode") return chain.toggleCode().run();
  if (id === "table")
    return chain.insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run();
  const applied = chain.run();
  if (applied) onImage();
  return applied;
}
type Menu = {
  from: number;
  to: number;
  query: string;
  key: string;
  x: number;
  y: number;
  selected: number;
  items: typeof commands;
};

/** Cursor-driven commands; the editor retains focus for typing and keyboard selection. */
export function useSlashMenu(
  editor: Editor | null,
  onImage: () => void,
  active: boolean,
  keyHandler: { current: (event: KeyboardEvent) => boolean },
) {
  const [menu, setMenu] = useState<Menu | null>(null);
  const current = useRef<Menu | null>(null);
  const dismissed = useRef<string | null>(null);
  const popup = useRef<HTMLDivElement>(null);
  const publish = (next: Menu | null) => {
    current.current = next;
    setMenu(next);
  };
  const choose = (id: CommandId) => {
    const value = current.current;
    if (!editor || !value || !editor.isEditable || editor.isDestroyed) return;
    publish(null);
    runSlashCommand(editor, id, { from: value.from, to: value.to }, onImage);
  };
  keyHandler.current = (event) => {
    const value = current.current;
    if (!value || event.isComposing) return false;
    if (event.key === "Escape") {
      dismissed.current = value.key;
      publish(null);
      return true;
    }
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      if (value.items.length)
        publish({
          ...value,
          selected:
            (value.selected + (event.key === "ArrowDown" ? 1 : -1) + value.items.length) %
            value.items.length,
        });
      return true;
    }
    if (event.key === "Enter" || event.key === "Tab") {
      const item = value.items[value.selected];
      if (item) {
        choose(item.id);
        return true;
      }
    }
    return false;
  };
  useEffect(() => {
    if (!editor || !active) {
      current.current = null;
      setMenu(null);
      return;
    }
    const update = () => {
      const { selection } = editor.state;
      if (
        !editor.isEditable ||
        !editor.isFocused ||
        editor.view.composing ||
        !selection.empty ||
        !selection.$from.parent.isTextblock
      ) {
        current.current = null;
        setMenu(null);
        return;
      }
      const before = selection.$from.parent.textBetween(
        0,
        selection.$from.parentOffset,
        "",
        "\uFFFC",
      );
      const match = /(?:^|\s)\/([a-zA-Z0-9 ]{0,24})$/.exec(before);
      if (!match) {
        current.current = null;
        setMenu(null);
        dismissed.current = null;
        return;
      }
      const query = match[1].toLowerCase();
      const from = selection.from - query.length - 1;
      const key = `${from}:${query}`;
      if (key === dismissed.current) return;
      const items = commands.filter((item) =>
        `${item.id} ${item.label} ${item.keywords}`.toLowerCase().includes(query.trim()),
      );
      const coords = editor.view.coordsAtPos(selection.from);
      const parent = editor.view.dom.closest(".pane__scroll")?.getBoundingClientRect();
      if (parent && (coords.bottom < parent.top || coords.top > parent.bottom)) {
        current.current = null;
        setMenu(null);
        return;
      }
      const height = Math.min(360, items.length * 54 + 72);
      const next: Menu = {
        from,
        to: selection.from,
        query,
        key,
        items,
        x: Math.max(12, Math.min(coords.left, window.innerWidth - 292)),
        y: Math.max(
          12,
          coords.bottom + height + 8 < window.innerHeight
            ? coords.bottom + 8
            : coords.top - height - 8,
        ),
        selected:
          current.current?.key === key
            ? Math.min(current.current.selected, Math.max(0, items.length - 1))
            : 0,
      };
      current.current = next;
      setMenu(next);
    };
    const close = () => {
      current.current = null;
      setMenu(null);
    };
    const outside = (event: PointerEvent) => {
      if (
        current.current &&
        !popup.current?.contains(event.target as Node) &&
        !editor.view.dom.contains(event.target as Node)
      ) {
        dismissed.current = current.current.key;
        close();
      }
    };
    editor.on("transaction", update);
    editor.on("focus", update);
    editor.on("blur", close);
    window.addEventListener("scroll", update, true);
    window.addEventListener("resize", update);
    document.addEventListener("pointerdown", outside);
    return () => {
      editor.off("transaction", update);
      editor.off("focus", update);
      editor.off("blur", close);
      window.removeEventListener("scroll", update, true);
      window.removeEventListener("resize", update);
      document.removeEventListener("pointerdown", outside);
    };
  }, [editor, active]);
  useEffect(() => {
    if (!editor || editor.isDestroyed) return;
    if (menu) {
      editor.view.dom.setAttribute("aria-controls", "chapter-slash-menu");
      const selected = menu.items[menu.selected];
      if (selected)
        editor.view.dom.setAttribute(
          "aria-activedescendant",
          `slash-command-${selected.id}`,
        );
      else editor.view.dom.removeAttribute("aria-activedescendant");
    } else {
      editor.view.dom.removeAttribute("aria-controls");
      editor.view.dom.removeAttribute("aria-activedescendant");
    }
  }, [editor, menu]);
  useEffect(() => {
    if (!menu) return;
    const list = popup.current?.querySelector<HTMLElement>('[role="listbox"]');
    const option = list?.querySelector<HTMLElement>('[aria-selected="true"]');
    if (!list || !option) return;
    if (option.offsetTop < list.scrollTop) list.scrollTop = option.offsetTop;
    else if (option.offsetTop + option.offsetHeight > list.scrollTop + list.clientHeight)
      list.scrollTop = option.offsetTop + option.offsetHeight - list.clientHeight;
  }, [menu]);
  return menu
    ? createPortal(
        <div ref={popup} className="slash-menu" style={{ left: menu.x, top: menu.y }}>
          <div className="slash-menu__title">Insert block</div>
          <div id="chapter-slash-menu" role="listbox" aria-label="Slash commands">
            {menu.items.map((item, index) => (
              <button
                type="button"
                id={`slash-command-${item.id}`}
                key={item.id}
                role="option"
                aria-selected={index === menu.selected}
                className="slash-menu__item"
                onMouseDown={(event) => event.preventDefault()}
                onMouseMove={() => publish({ ...menu, selected: index })}
                onClick={() => choose(item.id)}
              >
                <span className="slash-menu__icon">
                  {item.icon ? <Icon name={item.icon} /> : item.label}
                </span>
                <span>
                  <strong>{item.label}</strong>
                  <small>{item.detail}</small>
                </span>
              </button>
            ))}
            {!menu.items.length ? (
              <p className="slash-menu__empty">No matching commands</p>
            ) : null}
          </div>
          <div className="slash-menu__help">↑↓ choose · Enter insert · Esc close</div>
        </div>,
        document.body,
      )
    : null;
}
