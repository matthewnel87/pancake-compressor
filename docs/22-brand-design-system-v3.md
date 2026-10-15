# 22 — Brand direction & design tokens (required)

## North star
**Pancake Compressor: Squash file sizes, not image quality.** A fast, exceptionally simple image tool with warm personality. It should feel like a premium, carefully crafted Mac/Windows utility, not a childish game, bloated SaaS dashboard or generic AI-generated website.

### Composition of styles
- **60% calm modern utility:** uncluttered content hierarchy, strong typography, native-feeling actions, crisp icons.
- **30% tactile craft:** subtle pancake illustration, warm colour and gentle motion, but avoid fuzzy textured backgrounds behind text.
- **10% digi-cute:** small, delightful success moments. No persistent mascot animation while idle. No fake progress, confetti or sound by default.
- Elemental Folk is limited to one or two original, organic illustration details; omit collage and heavy florals in functional UI.

## Colour design tokens — proposals, not pre-certified contrast pairs
| Token | Light | Dark | Purpose |
|---|---|---|---|
| `--surface` | `#FFF9EF` | `#1D1917` | window background |
| `--surface-raised` | `#FFFFFF` | `#282220` | cards/table surfaces |
| `--ink` | `#2C241F` | `#FAF4E8` | body text |
| `--ink-muted` | `#665D55` | `#C7BBB0` | help text |
| `--accent` | `#AD6025` | `#F1B369` | primary action; verify foreground |
| `--butter` | `#F8D27D` | `#D3A44B` | illustration only |
| `--success` | `#206E53` | `#70D0A4` | confirmation with text |
| `--error` | `#A43230` | `#FF978F` | error with icon+text |

- Create both light and dark tokens, system theme detection, and a user override; save preference **locally**.
- Test **every text/background and focus combination** to WCAG 2.2 AA; adjust tokens where needed. Do not assume the proposed accent contrast is sufficient.
- Prefer bundled or system font stacks to avoid any runtime font download: `-apple-system, BlinkMacSystemFont, "Segoe UI", Inter, sans-serif` with Inter only when properly bundled and licensed. Use readable numeric tabular figures in progress/table results.

## Core UI tokens
- Base 8px spacing scale; content spacing 24px; card spacing 16px; row heights 44–52px. No tiny touch targets.
- Radii: 12px controls, 16px cards, 20px drop zone; avoid giant rounded SaaS pills everywhere.
- Thin 1px borders and restrained shadows; avoid backdrop blur that costs GPU and harms readability.
- Icon size generally 18–20px; descriptive labels, not icons alone.
- Responsive minimum window targeted at 800×560; preserve usable layout at smaller windows and 200% text scaling. Verify actual layouts.

## Copy style
- Neutral information first: "Compressing 4 of 12 images" rather than repeated jokes.
- Personality in microcopy: "Squashing your images…", "Nicely done. You saved 18 MB." One playful phrase per state maximum.
- No false claims: "visually lossless" cannot be claimed for lossy optimisation by default; "unlimited" means no software quotas, bounded by device capability.

## Do not
- Put an illustrated mascot in every table row, use endless bouncing effects, hard-code emojis as production icon assets, use inaccessible pastels as text, download Google Fonts at runtime, make the first-use journey more than one meaningful action, or use heavy front-end animation libraries without a measured need.

## Required design deliverables
`tokens.css` / Tailwind tokens; component catalogue; light/dark/200%-zoom screenshots; empty/working/success/error views; contrast test record; 3 brand sketch directions and selected final icon; brand asset licence record. Implementers must not present aesthetic placeholders as completed UX review.
