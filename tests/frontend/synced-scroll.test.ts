import { expect, test } from "bun:test";
import { proportionalScrollTop } from "../../src/features/workspace/useSyncedScroll";

test("proportional scroll maps the same ratio across different heights", () => {
  // Source halfway (500/1000) maps to half of target range (2000) = 1000.
  expect(proportionalScrollTop(500, 1000, 0, 2000)).toBe(1000);
  // Target at end maps source to its end.
  expect(proportionalScrollTop(2000, 2000, 0, 1000)).toBe(1000);
  // Top stays at top.
  expect(proportionalScrollTop(0, 1000, 999, 2000)).toBe(0);
});

test("proportional scroll returns null when no movement is needed", () => {
  // Already within 1px: no write, avoids feedback loops.
  expect(proportionalScrollTop(500, 1000, 1000.5, 2000)).toBeNull();
  // Neither side scrollable.
  expect(proportionalScrollTop(0, 0, 0, 1000)).toBeNull();
  expect(proportionalScrollTop(0, 1000, 0, 0)).toBeNull();
  // Clamped to range.
  expect(proportionalScrollTop(-50, 1000, 500, 1000)).toBe(0);
  expect(proportionalScrollTop(5000, 1000, 0, 400)).toBe(400);
});
