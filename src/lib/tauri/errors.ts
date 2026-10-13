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
