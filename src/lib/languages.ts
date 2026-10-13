/**
 * Readable names for BCP-47-ish language tags stored on a project.
 * Lookup is case-insensitive on the full tag, then on the primary subtag;
 * unknown tags fall back to the raw code so no language is ever hidden.
 */

const NAMES: Record<string, string> = {
  en: "English",
  my: "Myanmar",
  shn: "Shan",
  mnw: "Mon",
  kac: "Kachin",
  ksw: "S'gaw Karen",
  kar: "Karen",
  fr: "French",
  de: "German",
  es: "Spanish",
  it: "Italian",
  pt: "Portuguese",
  ru: "Russian",
  uk: "Ukrainian",
  zh: "Chinese",
  ja: "Japanese",
  ko: "Korean",
  th: "Thai",
  lo: "Lao",
  km: "Khmer",
  vi: "Vietnamese",
  id: "Indonesian",
  ms: "Malay",
  tl: "Tagalog",
  hi: "Hindi",
  bn: "Bengali",
  ta: "Tamil",
  te: "Telugu",
  ur: "Urdu",
  ne: "Nepali",
  si: "Sinhala",
  ar: "Arabic",
  fa: "Persian",
  he: "Hebrew",
  tr: "Turkish",
  el: "Greek",
  pl: "Polish",
  nl: "Dutch",
  sv: "Swedish",
  no: "Norwegian",
  da: "Danish",
  fi: "Finnish",
  hu: "Hungarian",
  cs: "Czech",
  ro: "Romanian",
};

/** Display name for a tag, or null when there is no tag to name. */
export function languageName(tag: string | null | undefined): string | null {
  if (!tag || tag.trim() === "") {
    return null;
  }
  const lowered = tag.trim().toLocaleLowerCase();
  return NAMES[lowered] ?? NAMES[lowered.split("-")[0]] ?? tag.trim();
}

/** One side of the direction label, e.g. `English (en)` or `Unknown`. */
function side(tag: string | null | undefined): string {
  if (!tag || tag.trim() === "") {
    return "Unknown";
  }
  const name = languageName(tag);
  return name && name !== tag.trim() ? `${name} (${tag.trim()})` : tag.trim();
}

/** Full translation direction, e.g. `English (en) → Myanmar (my)`. */
export function formatDirection(
  source: string | null | undefined,
  target: string,
): string {
  return `${side(source)} → ${side(target)}`;
}

/** Sorted suggestions for the language fields (`value` + readable label). */
export function languageSuggestions(): { value: string; label: string }[] {
  return Object.entries(NAMES)
    .map(([value, name]) => ({ value, label: `${name} (${value})` }))
    .sort((a, b) => a.label.localeCompare(b.label));
}
