import type { ThemePreference } from "../../lib/prefs";

export type AppearancePreferences = {
  theme: ThemePreference;
  textSize: number;
  onThemeChange: (theme: ThemePreference) => void;
  onTextSizeChange: (size: number) => void;
};

export function AppearanceSettings({
  theme,
  textSize,
  onThemeChange,
  onTextSizeChange,
}: AppearancePreferences) {
  return (
    <section aria-labelledby="appearance-heading">
      <h3 className="panel__heading" id="appearance-heading">
        Appearance
      </h3>
      <p className="dialog__muted">
        Changes apply immediately and are saved on this computer.
      </p>
      <div className="form-grid">
        <label className="form-field">
          <span>Theme</span>
          <select
            value={theme}
            onChange={(event) => {
              const value = event.target.value;
              if (value === "system" || value === "light" || value === "dark")
                onThemeChange(value);
            }}
          >
            <option value="system">Follow system</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </label>
        <label className="form-field">
          <span>Reading text size · {textSize}px</span>
          <input
            type="range"
            min={14}
            max={28}
            step={1}
            value={textSize}
            aria-valuetext={`${textSize} pixels`}
            onChange={(event) => onTextSizeChange(Number(event.target.value))}
          />
        </label>
      </div>
      <p className="dialog__muted">
        Text size applies to the original and translation. Shortcut: Cmd/Ctrl + or −.
      </p>
      <section
        className="appearance-preview"
        style={{ fontSize: textSize }}
        aria-label="Reading text preview"
      >
        <p>The next chapter begins here.</p>
        <p lang="my">မြန်မာစာကို ဖတ်ရှုရန်</p>
      </section>
    </section>
  );
}
