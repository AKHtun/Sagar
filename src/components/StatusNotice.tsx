import type { ReactNode } from "react";
import { useTranslation } from "../lib/i18n";
import { Icon } from "./Icon";

export type NoticeKind = "loading" | "success" | "error" | "info";

const KIND_LABEL: Record<NoticeKind, string> = {
  loading: "Working",
  success: "OK",
  error: "Error",
  info: "Note",
};

type StatusNoticeProps = {
  kind: NoticeKind;
  children: ReactNode;
  /** Only rendered for errors; keep failure feedback actionable. */
  onRetry?: () => void;
  retryLabel?: string;
  /** Optional extra action (e.g. an AI fix button) beside Retry. */
  action?: ReactNode;
};

/**
 * Reusable status feedback. Every state is communicated with text as well as
 * color (design rule), and errors use role="alert" so they are announced.
 */
export function StatusNotice({
  kind,
  children,
  onRetry,
  retryLabel = "Retry",
  action,
}: StatusNoticeProps) {
  const { t } = useTranslation();
  return (
    <div
      className={`notice notice--${kind}`}
      role={kind === "error" ? "alert" : "status"}
    >
      <span className="notice__kind">{t(KIND_LABEL[kind])}</span>
      <div className="notice__body">
        <span>{typeof children === "string" ? t(children) : children}</span>
        {kind === "error" && (onRetry || action) ? (
          <div className="notice__actions">
            {action}
            {onRetry ? (
              <button type="button" className="btn" onClick={onRetry}>
                <Icon name="book" />
                {t(retryLabel)}
              </button>
            ) : null}
          </div>
        ) : null}
      </div>
    </div>
  );
}
