/**
 * Light/dark theme choice. "system" follows the OS. The choice is stored in
 * this computer's local storage only (docs/22: save preference locally).
 */
export type ThemeChoice = "system" | "light" | "dark";

const KEY = "pancake-theme";

export function loadTheme(): ThemeChoice {
  try {
    const saved = window.localStorage.getItem(KEY);
    return saved === "light" || saved === "dark" ? saved : "system";
  } catch {
    return "system";
  }
}

export function applyTheme(choice: ThemeChoice): void {
  const root = document.documentElement;
  if (choice === "system") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", choice);
  try {
    if (choice === "system") window.localStorage.removeItem(KEY);
    else window.localStorage.setItem(KEY, choice);
  } catch {
    // Storage can be unavailable; the theme still applies for this session.
  }
}

export function nextTheme(choice: ThemeChoice): ThemeChoice {
  return choice === "system" ? "light" : choice === "light" ? "dark" : "system";
}

export const THEME_LABELS: Record<ThemeChoice, string> = {
  system: "Theme: match system",
  light: "Theme: light",
  dark: "Theme: dark",
};
