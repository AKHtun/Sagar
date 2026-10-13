import { useCallback, useEffect, useState } from "react";
import { Icon } from "../components/Icon";
import { LibraryScreen } from "../features/library/LibraryScreen";
import { WorkspaceFrame } from "../features/workspace/WorkspaceFrame";
import {
  loadTextZoom,
  loadTheme,
  saveTextZoom,
  saveTheme,
  type ThemePreference,
} from "../lib/prefs";
import type { ImportWarning } from "../types/project";

/**
 * App shell: navigation between the project library and the translation
 * workspace, plus global presentation preferences (theme, text zoom).
 */
export default function App() {
  const [openProject, setOpenProject] = useState<{
    projectId: string;
    importWarnings: ImportWarning[];
  } | null>(null);
  const [theme, setTheme] = useState<ThemePreference>(loadTheme);
  const [zoom, setZoom] = useState<number>(loadTextZoom);

  // Apply presentation preferences to the document. The "system" theme is
  // resolved in CSS via prefers-color-scheme.
  useEffect(() => {
    const root = document.documentElement;
    root.dataset.theme = theme;
    root.style.setProperty("--text-book", `${zoom}px`);
  }, [theme, zoom]);

  const cycleTheme = useCallback(() => {
    const order: ThemePreference[] = ["system", "light", "dark"];
    const next = order[(order.indexOf(loadTheme()) + 1) % order.length];
    setTheme(next);
    saveTheme(next);
  }, []);

  const zoomIn = useCallback(() => {
    setZoom((current) => {
      const next = Math.min(28, current + 2);
      saveTextZoom(next);
      return next;
    });
  }, []);

  const zoomOut = useCallback(() => {
    setZoom((current) => {
      const next = Math.max(14, current - 2);
      saveTextZoom(next);
      return next;
    });
  }, []);

  // Global shortcuts: Cmd/Ctrl+O opens a book, Cmd/Ctrl+= / - adjusts
  // reading text size. Editor shortcuts (undo, paste) stay native.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!(event.metaKey || event.ctrlKey)) {
        return;
      }
      const key = event.key.toLowerCase();
      if (key === "o" && openProject === null) {
        event.preventDefault();
        document.querySelector<HTMLButtonElement>("[data-open-epub]")?.click();
      } else if (event.key === "=" || event.key === "+") {
        event.preventDefault();
        zoomIn();
      } else if (event.key === "-") {
        event.preventDefault();
        zoomOut();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [openProject, zoomIn, zoomOut]);

  if (openProject) {
    return (
      <WorkspaceFrame
        projectId={openProject.projectId}
        importWarnings={openProject.importWarnings}
        onBackToLibrary={() => setOpenProject(null)}
      />
    );
  }
  return (
    <div className="app-with-controls">
      <div className="app-controls" role="toolbar" aria-label="Appearance">
        <div className="app-brand">
          <span>
            <Icon name="book" size={21} />
          </span>{" "}
          Sagar <span className="app-brand__descriptor">Translation studio</span>
        </div>
        <button
          type="button"
          className="icon-button"
          onClick={cycleTheme}
          aria-label={`Appearance: ${theme}`}
          title={`Appearance: ${theme}. Click to change.`}
        >
          <Icon
            name={theme === "system" ? "monitor" : theme === "light" ? "sun" : "moon"}
          />
        </button>
        <button
          type="button"
          className="icon-button"
          onClick={zoomIn}
          aria-label="Increase text size"
          title="Increase text size (Cmd/Ctrl +)"
        >
          <Icon name="plus" />
        </button>
        <button
          type="button"
          className="icon-button"
          onClick={zoomOut}
          aria-label="Decrease text size"
          title="Decrease text size (Cmd/Ctrl −)"
        >
          <Icon name="minus" />
        </button>
      </div>
      <LibraryScreen
        onOpenProject={(projectId, importWarnings) =>
          setOpenProject({ projectId, importWarnings })
        }
      />
    </div>
  );
}
