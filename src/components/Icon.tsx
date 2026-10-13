const paths = {
  book: "M4 4h6a3 3 0 0 1 2 1 3 3 0 0 1 2-1h6v15h-6a3 3 0 0 0-2 1 3 3 0 0 0-2-1H4V4Zm8 1v15",
  trash: "M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7M14 10v7",
  plus: "M12 5v14M5 12h14",
  minus: "M5 12h14",
  search: "M21 21l-5-5M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Z",
  upload: "M12 16V3M7 8l5-5 5 5M4 15v6h16v-6",
  download: "M12 3v13M7 11l5 5 5-5M4 17v4h16v-4",
  left: "M15 5l-7 7 7 7",
  right: "M9 5l7 7-7 7",
  sidebar: "M3 4h18v16H3V4Zm6 0v16",
  check: "M5 12l4 4L19 6",
  keep: "M5 4h14v16H5V4Zm4 5h6M9 13h6",
  sun: "M12 8a4 4 0 1 1 0 8 4 4 0 0 1 0-8ZM12 2v2M12 20v2M2 12h2M20 12h2M5 5l1 1M18 18l1 1M5 19l1-1M18 6l1-1",
  moon: "M21 13A9 9 0 1 1 11 3a7 7 0 0 0 10 10Z",
  monitor: "M3 3h18v14H3V3Zm9 14v4M8 21h8",
  bold: "M6 4h7a4 4 0 0 1 0 8H6V4Zm0 8h8a4 4 0 0 1 0 8H6v-8Z",
  italic: "M10 4h9M5 20h9M15 4 9 20",
  underline: "M6 3v8a6 6 0 0 0 12 0V3M4 21h16",
  strike: "M17 5c-2-3-10-2-10 2 0 2 2 3 5 4M4 12h16M7 19c3 3 10 2 10-2 0-1-1-2-3-3",
  undo: "M9 5 4 10l5 5M4 10h10a6 6 0 0 1 6 6v3",
  redo: "M15 5l5 5-5 5M20 10H10a6 6 0 0 0-6 6v3",
  image: "M3 3h18v18H3V3Zm0 14 6-6 5 5 3-3 4 4M16 7h.01",
  list: "M9 5h12M9 12h12M9 19h12M3 5h1M3 12h1M3 19h1",
  numbered: "M10 5h11M10 12h11M10 19h11M3 3h2v5M3 10h3v2l-3 3h3M3 18h3v4H3",
  code: "M8 6 2 12l6 6M16 6l6 6-6 6M14 3l-4 18",
  edit: "M4 20h4L20 8l-4-4L4 16v4ZM14 6l4 4",
  rowAdd: "M3 3h18v11H3V3Zm0 6h18M12 17v6M9 20h6",
  columnAdd: "M3 3h11v18H3V3Zm6 0v18M17 12h6M20 9v6",
  rowRemove: "M3 3h18v11H3V3Zm0 6h18M9 20h6",
  columnRemove: "M3 3h11v18H3V3Zm6 0v18M17 12h6",
  table: "M3 3h18v18H3V3Zm0 6h18M3 15h18M9 3v18M15 3v18",
  close: "M6 6l12 12M6 18 18 6",
} as const;
export type IconName = keyof typeof paths;
export function Icon({ name, size = 18 }: { name: IconName; size?: number }) {
  return (
    <svg
      className="icon"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      <path d={paths[name]} />
    </svg>
  );
}
