import { expect, test } from "bun:test";

const css = await Bun.file(new URL("../../src/styles/tokens.css", import.meta.url)).text();
const palettes = [...css.matchAll(/(?:^:root|\[data-theme="dark"\])\s*\{([^}]+)\}/gm)];

function luminance(hex: string) {
  const channels = hex.match(/[a-f\d]{2}/gi)!.map((part) => {
    const channel = Number.parseInt(part, 16) / 255;
    return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  });
  return channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
}

test("light and dark text, disabled labels and status messages meet 4.5:1 contrast", () => {
  expect(palettes).toHaveLength(2);
  for (const [, body] of palettes) {
    const colors = Object.fromEntries([...body.matchAll(/--color-([\w-]+):\s*(#[a-f\d]{6})/gi)].map(([, name, value]) => [name, value]));
    const pairs = [
      ...["bg", "surface", "surface-hover", "surface-selected"].flatMap((surface) => [["text", surface], ["text-muted", surface]]),
      ["accent-text", "accent"],
      ...["success", "warning", "error", "info"].map((status) => [status, `${status}-bg`]),
    ];
    for (const [foreground, background] of pairs) {
      const a = luminance(colors[foreground]);
      const b = luminance(colors[background]);
      expect((Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05)).toBeGreaterThanOrEqual(4.5);
    }
  }
});
