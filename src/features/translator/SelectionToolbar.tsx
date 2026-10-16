import { TextSelection } from "@tiptap/pm/state";
import type { Editor } from "@tiptap/react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Icon, type IconName } from "../../components/Icon";
import { toggleBookInline } from "./inlineFormatting";

const formats: { label: string; mark: string; icon: IconName }[] = [
  { label: "Bold", mark: "bold", icon: "bold" },
  { label: "Italic", mark: "italic", icon: "italic" },
  { label: "Underline", mark: "underline", icon: "underline" },
  { label: "Strikethrough", mark: "strike", icon: "strike" },
  { label: "Highlight", mark: "book_mark", icon: "highlight" },
  { label: "Inline code", mark: "code", icon: "code" },
  { label: "Superscript", mark: "book_sup", icon: "superscript" },
  { label: "Subscript", mark: "book_sub", icon: "subscript" },
];

/** The Rust document validator remains authoritative; reject unsafe input early. */
export function isEditorLink(value: string) {
  return (
    value.length > 0 &&
    !/[\s\\<>]/u.test(value) &&
    !Array.from(value).some(
      (char) => char.charCodeAt(0) < 32 || char.charCodeAt(0) === 127,
    ) &&
    !value.startsWith("/") &&
    (/^(https?:\/\/|mailto:)/i.test(value) || !value.includes(":"))
  );
}

type Position = { x: number; y: number };

/** Selection-only controls, outside the scrolling pane so they never get clipped. */
export function SelectionToolbar({
  editor,
  active,
}: {
  editor: Editor;
  active: boolean;
}) {
  const popup = useRef<HTMLFieldSetElement>(null);
  const [position, setPosition] = useState<Position | null>(null);
  const [link, setLink] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const dismissed = useRef<string | null>(null);
  const input = useRef<HTMLInputElement>(null);
  const updatePosition = useRef(() => {});
  const visible = position !== null;

  useEffect(() => {
    if (!active) {
      setPosition(null);
      setLink(null);
      return;
    }
    let frame = 0;
    let previousSelection = editor.state.selection;
    let previousDocument = editor.state.doc;
    const update = () => {
      if (editor.isDestroyed) return;
      const { selection } = editor.state;
      const key = `${selection.from}:${selection.to}`;
      const hasFocus =
        editor.isFocused || popup.current?.contains(document.activeElement);
      if (
        !editor.isEditable ||
        !hasFocus ||
        editor.view.composing ||
        !(selection instanceof TextSelection) ||
        selection.empty ||
        !editor.state.doc.textBetween(selection.from, selection.to).trim() ||
        dismissed.current === key
      ) {
        setPosition(null);
        setLink(null);
        if (selection.empty) dismissed.current = null;
        return;
      }
      dismissed.current = null;
      const start = editor.view.coordsAtPos(selection.from);
      const end = editor.view.coordsAtPos(selection.to);
      const pane = editor.view.dom.closest(".pane__scroll")?.getBoundingClientRect();
      const top = Math.max(start.top, pane?.top ?? 0);
      const bottom = Math.min(end.bottom, pane?.bottom ?? window.innerHeight);
      if (bottom < top || bottom < 0 || top > window.innerHeight) {
        setPosition(null);
        setLink(null);
        return;
      }
      const width = popup.current?.offsetWidth || 354;
      const height = popup.current?.offsetHeight || 46;
      setPosition({
        x: Math.max(
          8,
          Math.min((start.left + end.right - width) / 2, window.innerWidth - width - 8),
        ),
        y: Math.max(
          8,
          Math.min(
            top - height - 8 >= 8 ? top - height - 8 : bottom + 8,
            window.innerHeight - height - 8,
          ),
        ),
      });
    };
    updatePosition.current = update;
    const deferUpdate = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(update);
    };
    const onTransaction = () => {
      // Focus/blur also dispatch transactions. Only invalidate the link form
      // when its text range or document actually changed.
      if (
        !editor.state.selection.eq(previousSelection) ||
        editor.state.doc !== previousDocument
      )
        setLink(null);
      previousSelection = editor.state.selection;
      previousDocument = editor.state.doc;
      deferUpdate();
    };
    const outside = (event: PointerEvent) => {
      if (
        !popup.current?.contains(event.target as Node) &&
        !editor.view.dom.contains(event.target as Node)
      ) {
        dismissed.current = `${editor.state.selection.from}:${editor.state.selection.to}`;
        setPosition(null);
        setLink(null);
      }
    };
    const keyboard = (event: KeyboardEvent) => {
      if (event.isComposing || !popup.current) return;
      const inPopup = popup.current.contains(event.target as Node);
      const inEditor = editor.view.dom.contains(event.target as Node);
      if (!inPopup && !inEditor) return;
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        dismissed.current = `${editor.state.selection.from}:${editor.state.selection.to}`;
        setPosition(null);
        setLink(null);
        editor.commands.focus();
      } else if (event.key === "Tab" && inEditor && !event.shiftKey) {
        event.preventDefault();
        popup.current.querySelector<HTMLButtonElement>("button:not(:disabled)")?.focus();
      } else if (
        inPopup &&
        ["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key) &&
        (event.target as HTMLElement).tagName === "BUTTON"
      ) {
        const buttons = Array.from(
          popup.current.querySelectorAll<HTMLButtonElement>("button:not(:disabled)"),
        );
        const index = buttons.indexOf(event.target as HTMLButtonElement);
        const next =
          event.key === "Home"
            ? 0
            : event.key === "End"
              ? buttons.length - 1
              : (index + (event.key === "ArrowRight" ? 1 : -1) + buttons.length) %
                buttons.length;
        event.preventDefault();
        buttons[next]?.focus();
      }
    };
    editor.on("transaction", onTransaction);
    editor.on("focus", deferUpdate);
    editor.on("blur", deferUpdate);
    window.addEventListener("scroll", deferUpdate, true);
    window.addEventListener("resize", deferUpdate);
    document.addEventListener("pointerdown", outside);
    document.addEventListener("keydown", keyboard, true);
    editor.view.dom.addEventListener("compositionstart", deferUpdate);
    editor.view.dom.addEventListener("compositionend", deferUpdate);
    deferUpdate();
    return () => {
      updatePosition.current = () => {};
      cancelAnimationFrame(frame);
      editor.off("transaction", onTransaction);
      editor.off("focus", deferUpdate);
      editor.off("blur", deferUpdate);
      window.removeEventListener("scroll", deferUpdate, true);
      window.removeEventListener("resize", deferUpdate);
      document.removeEventListener("pointerdown", outside);
      document.removeEventListener("keydown", keyboard, true);
      editor.view.dom.removeEventListener("compositionstart", deferUpdate);
      editor.view.dom.removeEventListener("compositionend", deferUpdate);
    };
  }, [editor, active]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: mount/form/error changes alter the measured popup dimensions.
  useLayoutEffect(() => {
    if (link !== null) input.current?.focus();
    updatePosition.current();
  }, [link, error, visible]);

  const openLink = () => {
    setError(null);
    setLink((editor.getAttributes("link").href as string | undefined) ?? "");
  };
  const linkHasBookAttributes = Object.entries(editor.getAttributes("link")).some(
    ([name, value]) =>
      !["href", "target", "rel"].includes(name) &&
      value !== null &&
      (typeof value !== "object" || Object.keys(value).length > 0),
  );
  const applyLink = () => {
    const href = link?.trim() ?? "";
    if (!isEditorLink(href)) {
      setError(
        "Use https://…, mailto:…, a book-relative path, or #anchor. Encode spaces as %20.",
      );
      input.current?.focus();
      return;
    }
    if (
      !editor
        .chain()
        .focus()
        .extendMarkRange("link")
        .setLink({ href, target: null, rel: null })
        .run()
    ) {
      setError("This selection cannot contain a link. Select text outside inline code.");
      return;
    }
    setLink(null);
  };

  if (!active || !position || editor.isDestroyed || !editor.isEditable) return null;
  return createPortal(
    <fieldset
      ref={popup}
      aria-label="Selection tools"
      className="selection-popup"
      style={{ left: position.x, top: position.y }}
      onBlur={() => requestAnimationFrame(() => updatePosition.current())}
    >
      {link === null ? (
        <div
          role="toolbar"
          aria-label="Selected text formatting"
          className="selection-popup__tools"
        >
          {formats.map(({ label, mark, icon }) => (
            <button
              key={mark}
              type="button"
              className="icon-button"
              title={label}
              aria-label={label}
              aria-pressed={editor.isActive(mark)}
              disabled={
                mark.startsWith("book_")
                  ? !toggleBookInline(editor, mark, false)
                  : !editor.can().toggleMark(mark)
              }
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => {
                if (mark.startsWith("book_")) {
                  toggleBookInline(editor, mark);
                  editor.commands.focus();
                } else editor.chain().focus().toggleMark(mark).run();
              }}
            >
              <Icon name={icon} />
            </button>
          ))}
          <span className="selection-popup__divider" />
          <button
            type="button"
            className="icon-button"
            title="Edit link"
            aria-label="Edit link"
            aria-pressed={editor.isActive("link")}
            disabled={!editor.can().setLink({ href: "#anchor" })}
            onMouseDown={(event) => event.preventDefault()}
            onClick={openLink}
          >
            <Icon name="link" />
          </button>
        </div>
      ) : (
        <form
          className="selection-popup__link"
          onSubmit={(event) => {
            event.preventDefault();
            applyLink();
          }}
        >
          <div className="selection-popup__link-heading">
            <label htmlFor="selection-link-url">Link address</label>
            <button
              type="button"
              className="icon-button"
              title="Cancel link editing"
              aria-label="Cancel link editing"
              onClick={() => {
                setLink(null);
                editor.commands.focus();
              }}
            >
              <Icon name="close" />
            </button>
          </div>
          <input
            ref={input}
            id="selection-link-url"
            value={link}
            placeholder="https://… or #anchor"
            aria-invalid={!!error}
            aria-describedby={error ? "selection-link-error" : undefined}
            onInput={(event) => {
              setLink(event.currentTarget.value);
              setError(null);
            }}
          />
          {error ? (
            <p id="selection-link-error" role="alert">
              {error}
            </p>
          ) : null}
          <div className="selection-popup__link-actions">
            <button
              type="button"
              className="btn btn--small"
              disabled={!editor.isActive("link") || linkHasBookAttributes}
              title={
                linkHasBookAttributes
                  ? "This link carries book attributes. Edit its address to preserve them."
                  : "Remove link"
              }
              onClick={() => {
                if (linkHasBookAttributes) return;
                editor.chain().focus().extendMarkRange("link").unsetLink().run();
                setLink(null);
              }}
            >
              <Icon name="unlink" />
              Remove link
            </button>
            <button type="submit" className="btn btn--primary btn--small">
              <Icon name="save" />
              Save
            </button>
          </div>
        </form>
      )}
    </fieldset>,
    document.body,
  );
}
