import type { KeyboardEvent, PointerEvent } from "react";
import { useCallback, useRef } from "react";

type SplitterProps = {
  /** Accessible name, e.g. "Resize chapters sidebar". */
  label: string;
  orientation: "vertical" | "horizontal";
  /** Current size in px of the leading pane. */
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
  /** For a vertical splitter, true when the value counts from the right edge. */
  invert?: boolean;
};

const STEP = 24;

/**
 * Pointer- and keyboard-operable splitter between panes (design rule:
 * resizing works without a mouse and never hides required controls).
 */
export function Splitter({
  label,
  orientation,
  value,
  min,
  max,
  onChange,
  invert,
}: SplitterProps) {
  const dragState = useRef<{ startPointer: number; startValue: number } | null>(null);

  const applyDelta = useCallback(
    (delta: number) => {
      const signed = invert ? -delta : delta;
      onChange(Math.min(max, Math.max(min, value + signed)));
    },
    [invert, max, min, onChange, value],
  );

  const onPointerDown = useCallback(
    (event: PointerEvent<HTMLDivElement>) => {
      dragState.current = {
        startPointer: orientation === "vertical" ? event.clientX : event.clientY,
        startValue: value,
      };
      event.currentTarget.setPointerCapture(event.pointerId);
    },
    [orientation, value],
  );

  const onPointerMove = useCallback(
    (event: PointerEvent<HTMLDivElement>) => {
      const drag = dragState.current;
      if (!drag) {
        return;
      }
      const pointer = orientation === "vertical" ? event.clientX : event.clientY;
      const delta = (pointer - drag.startPointer) * (invert ? -1 : 1);
      onChange(Math.min(max, Math.max(min, drag.startValue + delta)));
    },
    [invert, max, min, onChange, orientation],
  );

  const onPointerUp = useCallback(() => {
    dragState.current = null;
  }, []);

  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLDivElement>) => {
      const negative = invert ? "ArrowRight" : "ArrowLeft";
      const positive = invert ? "ArrowLeft" : "ArrowRight";
      if (event.key === negative) {
        event.preventDefault();
        applyDelta(-STEP);
      } else if (event.key === positive) {
        event.preventDefault();
        applyDelta(STEP);
      } else if (event.key === "Home") {
        event.preventDefault();
        onChange(min);
      } else if (event.key === "End") {
        event.preventDefault();
        onChange(max);
      }
    },
    [applyDelta, invert, max, min, onChange],
  );

  return (
    <div
      role="separator"
      aria-orientation={orientation}
      aria-label={label}
      aria-valuenow={Math.round(value)}
      aria-valuemin={min}
      aria-valuemax={max}
      tabIndex={0}
      className={`splitter splitter--${orientation}`}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onKeyDown={onKeyDown}
    />
  );
}
