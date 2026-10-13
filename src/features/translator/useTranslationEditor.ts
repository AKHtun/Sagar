import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { saveTranslation } from "../../lib/tauri/commands";
import type { BlockStatus, ChapterContent } from "../../types/chapter";
import { hasContent, plainFragment } from "./richText";

export type EditorBlock = {
  text: string;
  status: BlockStatus;
  revision: number;
  /** Local text differs from the last acknowledged save. */
  dirty: boolean;
  error: string | null;
  conflict: boolean;
};

export type SaveIndicator = "saved" | "unsaved" | "saving" | "error";

const DEBOUNCE_MS = 600;

/**
 * Owns per-block translation drafts, debounced autosave with optimistic
 * revision checking, and explicit status actions. Local text is never
 * discarded on failure; conflicts keep the local draft and surface a
 * deliberate resolution.
 */
export function useTranslationEditor(content: ChapterContent | null) {
  const [blocks, setBlocks] = useState<Map<string, EditorBlock>>(new Map());
  const [chapterStatus, setChapterStatus] = useState<string | null>(null);
  const [inflightCount, setInflightCount] = useState(0);
  const timers = useRef(new Map<string, number>());
  const inflight = useRef(new Map<string, Promise<void>>());
  const latest = useRef(new Map<string, EditorBlock>());

  const reset = useCallback((content: ChapterContent | null) => {
    for (const timer of timers.current.values()) {
      window.clearTimeout(timer);
    }
    timers.current.clear();
    const next = new Map<string, EditorBlock>();
    for (const block of content?.blocks ?? []) {
      if (block.supportStatus !== "supported") {
        continue;
      }
      next.set(block.id, {
        text: block.targetHtml ?? plainFragment(block.targetPlainText ?? ""),
        status: block.status,
        revision: block.revision,
        dirty: false,
        error: null,
        conflict: false,
      });
    }
    latest.current = next;
    setBlocks(new Map(next));
    setChapterStatus(null);
  }, []);

  useEffect(() => {
    reset(content);
    return () => {
      for (const timer of timers.current.values()) {
        window.clearTimeout(timer);
      }
      timers.current.clear();
    };
  }, [content, reset]);

  const update = useCallback((blockId: string, patch: Partial<EditorBlock>) => {
    const current = latest.current.get(blockId);
    if (!current) {
      return;
    }
    const nextBlock = { ...current, ...patch };
    latest.current.set(blockId, nextBlock);
    setBlocks(new Map(latest.current));
  }, []);

  const scheduleRef = useRef<
    (blockId: string, text: string, status: BlockStatus) => void
  >(() => {});

  const performSave = useCallback(
    (blockId: string, text: string, status: BlockStatus) => {
      const pending = inflight.current.get(blockId);
      if (pending) {
        void pending.then(() => {
          const current = latest.current.get(blockId);
          if (current?.dirty && !current.conflict && !current.error) {
            performSave(blockId, current.text, current.status);
          }
        });
        return;
      }
      const current = latest.current.get(blockId);
      if (!current) {
        return;
      }
      const expectedRevision = current.revision;
      const request = saveTranslation(
        blockId,
        !hasContent(text) ? null : text,
        status,
        expectedRevision,
      )
        .then((result) => {
          const saved = latest.current.get(blockId);
          update(blockId, {
            revision: result.revision,
            status:
              saved?.dirty && (saved.text !== text || saved.status !== status)
                ? saved.status
                : result.status,
            error: null,
            conflict: false,
            // Still dirty if the user typed while this save was in flight.
            dirty:
              saved !== undefined && (saved.text !== text || saved.status !== status),
          });
          setChapterStatus(result.chapterStatus);
          if (
            saved !== undefined &&
            (saved.text !== text || saved.status !== status) &&
            !timers.current.has(blockId)
          ) {
            scheduleRef.current(blockId, saved.text, saved.status);
          }
        })
        .catch((error: unknown) => {
          const code =
            typeof error === "object" && error !== null && "code" in error
              ? String((error as { code: unknown }).code)
              : "unknown";
          update(blockId, {
            error:
              code === "revision_conflict"
                ? "A newer saved version exists. Reload to continue from the saved content."
                : error instanceof Error
                  ? error.message
                  : "The block could not be saved.",
            conflict: code === "revision_conflict",
          });
        })
        .finally(() => {
          inflight.current.delete(blockId);
          setInflightCount(inflight.current.size);
        });
      inflight.current.set(blockId, request);
      setInflightCount(inflight.current.size);
    },
    [update],
  );

  const schedule = useCallback(
    (blockId: string, text: string, status: BlockStatus) => {
      const existing = timers.current.get(blockId);
      if (existing !== undefined) {
        window.clearTimeout(existing);
      }
      const timer = window.setTimeout(() => {
        timers.current.delete(blockId);
        performSave(blockId, text, status);
      }, DEBOUNCE_MS);
      timers.current.set(blockId, timer);
    },
    [performSave],
  );

  useEffect(() => {
    scheduleRef.current = schedule;
  }, [schedule]);

  const setText = useCallback(
    (blockId: string, text: string) => {
      const current = latest.current.get(blockId);
      if (!current || current.conflict) {
        return;
      }
      const nextStatus: BlockStatus = "draft";
      update(blockId, { text, dirty: true, status: nextStatus, error: null });
      schedule(blockId, text, "draft");
    },
    [schedule, update],
  );

  const markReviewed = useCallback(
    (blockId: string) => {
      const current = latest.current.get(blockId);
      if (!current || !hasContent(current.text) || current.conflict) {
        return;
      }
      const timer = timers.current.get(blockId);
      if (timer !== undefined) {
        window.clearTimeout(timer);
        timers.current.delete(blockId);
      }
      update(blockId, { error: null, dirty: true, status: "reviewed" });
      performSave(blockId, current.text, "reviewed");
    },
    [performSave, update],
  );

  const keepOriginal = useCallback(
    (blockId: string) => {
      const current = latest.current.get(blockId);
      if (!current || current.conflict) {
        return;
      }
      const timer = timers.current.get(blockId);
      if (timer !== undefined) {
        window.clearTimeout(timer);
        timers.current.delete(blockId);
      }
      update(blockId, { text: "", error: null, dirty: true, status: "skipped" });
      performSave(blockId, "", "skipped");
    },
    [performSave, update],
  );

  const translateAgain = useCallback(
    (blockId: string, sourceText: string) => {
      const current = latest.current.get(blockId);
      if (!current || current.conflict) {
        return;
      }
      const timer = timers.current.get(blockId);
      if (timer !== undefined) {
        window.clearTimeout(timer);
        timers.current.delete(blockId);
      }
      update(blockId, {
        text: plainFragment(sourceText),
        dirty: true,
        status: "draft",
        error: null,
      });
      performSave(blockId, plainFragment(sourceText), "draft");
    },
    [performSave, update],
  );

  const copyOriginal = useCallback(
    (blockId: string, sourceText: string) => {
      setText(blockId, plainFragment(sourceText));
    },
    [setText],
  );

  const retry = useCallback(
    (blockId: string) => {
      const current = latest.current.get(blockId);
      if (!current) {
        return;
      }
      update(blockId, { error: null });
      performSave(
        blockId,
        current.text,
        current.status === "reviewed" && hasContent(current.text) ? "reviewed" : "draft",
      );
    },
    [performSave, update],
  );

  /** Saves every dirty block immediately; resolves when writes settle. */
  const flushAll = useCallback(async (): Promise<boolean> => {
    for (const [blockId, block] of latest.current) {
      if (block.dirty && !block.conflict) {
        const timer = timers.current.get(blockId);
        if (timer !== undefined) {
          window.clearTimeout(timer);
          timers.current.delete(blockId);
        }
        performSave(blockId, block.text, block.status);
      }
    }
    while (inflight.current.size > 0 || timers.current.size > 0) {
      await Promise.allSettled([...inflight.current.values()]);
      for (const [id, timer] of timers.current) {
        window.clearTimeout(timer);
        timers.current.delete(id);
        const block = latest.current.get(id);
        if (block?.dirty && !block.error && !block.conflict) {
          performSave(id, block.text, block.status);
        }
      }
    }
    let hasError = false;
    for (const block of latest.current.values()) {
      if (block.error !== null) {
        hasError = true;
      }
    }
    return !hasError;
  }, [performSave]);

  const indicator: SaveIndicator = useMemo(() => {
    let hasError = false;
    let hasDirty = false;
    for (const block of blocks.values()) {
      if (block.error !== null) {
        hasError = true;
      }
      if (block.dirty) {
        hasDirty = true;
      }
    }
    if (hasError) {
      return "error";
    }
    if (inflightCount > 0) {
      return "saving";
    }
    return hasDirty ? "unsaved" : "saved";
  }, [blocks, inflightCount]);

  const hasErrors = indicator === "error";

  return {
    blocks,
    chapterStatus,
    indicator,
    hasErrors,
    setText,
    markReviewed,
    keepOriginal,
    translateAgain,
    copyOriginal,
    retry,
    flushAll,
  };
}
