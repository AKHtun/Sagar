/**
 * Structured error shape mirrored from src-tauri/src/error.rs.
 * Rust command failures reject with `{ code, message, details? }`; anything
 * else (e.g. a plain string or a WebView-level failure) is wrapped as
 * `unknown`.
 */
export class CommandError extends Error {
  readonly code: string;
  readonly details?: unknown;

  constructor(code: string, message: string, details?: unknown) {
    super(message);
    this.name = "CommandError";
    this.code = code;
    this.details = details;
  }
}

export function toCommandError(error: unknown): CommandError {
  if (error instanceof CommandError) {
    return error;
  }
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    "message" in error
  ) {
    const raw = error as { code: unknown; message: unknown; details?: unknown };
    return new CommandError(String(raw.code), String(raw.message), raw.details);
  }
  if (typeof error === "string" && error.length > 0) {
    return new CommandError("unknown", error);
  }
  return new CommandError("unknown", "An unexpected error occurred.");
}

/**
 * True for transient SQLite lock contention ("database is locked") that is
 * safe to retry with the same arguments. The Rust backend already waits and
 * retries internally, so this should rarely fire — it covers the residual
 * overlap window (e.g. an autosave racing a chapter load on another thread).
 */
export function isBusyError(error: unknown): boolean {
  if (error instanceof CommandError) {
    if (error.code === "storage_busy") {
      return true;
    }
    if (
      typeof error.details === "object" &&
      error.details !== null &&
      "retryable" in error.details &&
      (error.details as { retryable: unknown }).retryable === true
    ) {
      return true;
    }
    return /database is (locked|busy)/i.test(error.message);
  }
  if (typeof error === "object" && error !== null && "code" in error) {
    const code = String((error as { code: unknown }).code);
    if (code === "storage_busy") {
      return true;
    }
  }
  return error instanceof Error && /database is (locked|busy)/i.test(error.message);
}
