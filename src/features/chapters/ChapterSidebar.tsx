import type { KeyboardEvent } from "react";
import type { ChapterSummary } from "../../types/project";

type ChapterSidebarProps = {
  chapters: ChapterSummary[];
  activeChapterId: string | null;
  /** Live status of the active chapter, derived from block saves. */
  chapterStatusOverride: string | null;
  onSelect: (chapterId: string) => void;
};

/**
 * Chapter list in OPF spine order. A chapter here is one spine resource;
 * table-of-contents anchors within a resource are not separate chapters.
 */
export function ChapterSidebar({
  chapters,
  activeChapterId,
  chapterStatusOverride,
  onSelect,
}: ChapterSidebarProps) {
  return (
    <nav className="chapter-nav" aria-label="Chapters">
      <h2 className="chapter-nav__heading">Chapters</h2>
      {chapters.length === 0 ? (
        <p className="chapter-nav__empty">No chapters found.</p>
      ) : (
        <ol className="chapter-list" onKeyDown={onListKeyDown}>
          {chapters.map((chapter) => {
            const isActive = chapter.id === activeChapterId;
            return (
              <li key={chapter.id}>
                <button
                  type="button"
                  className="chapter-list__select"
                  aria-current={isActive ? "true" : undefined}
                  tabIndex={isActive ? 0 : -1}
                  onClick={() => onSelect(chapter.id)}
                >
                  <span className="chapter-list__index">
                    {String(chapter.spineIndex + 1).padStart(2, "0")}
                  </span>
                  <span>
                    <span className="chapter-list__title">{chapter.title}</span>
                    <span className="chapter-list__status">
                      {statusLabel(chapter, isActive ? chapterStatusOverride : null)}
                    </span>
                  </span>
                </button>
              </li>
            );
          })}
        </ol>
      )}
    </nav>
  );
}

function onListKeyDown(event: KeyboardEvent<HTMLOListElement>) {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") {
    return;
  }
  event.preventDefault();
  const rows = Array.from(
    event.currentTarget.querySelectorAll<HTMLButtonElement>(".chapter-list__select"),
  );
  const currentIndex = rows.findIndex((row) => row.tabIndex === 0);
  const next =
    event.key === "ArrowDown"
      ? Math.min(rows.length - 1, currentIndex + 1)
      : Math.max(0, currentIndex - 1);
  const target = rows[next];
  if (target) {
    target.focus();
    target.click();
  }
}

function statusLabel(chapter: ChapterSummary, liveStatus: string | null): string {
  const status =
    liveStatus === "completed"
      ? "Completed"
      : liveStatus === "in_progress"
        ? "In progress"
        : liveStatus === "not_started"
          ? "Not started"
          : chapter.status === "not_started"
            ? "Not started"
            : chapter.status === "in_progress"
              ? "In progress"
              : "Completed";
  return status;
}
