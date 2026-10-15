/**
 * TEMPORARY placeholder mark: a simple three-layer pancake stack.
 *
 * This is not the brand logo. docs/27-logo-and-brand-asset-production.md
 * requires three design routes and maintainer sign-off before a final icon
 * ships. Decorative only, so it is hidden from assistive technology.
 */
export function PancakeMark({ className = "" }: { className?: string }) {
  return (
    <svg
      className={className}
      viewBox="0 0 64 64"
      aria-hidden="true"
      focusable="false"
      data-placeholder="pancake-mark"
    >
      <ellipse cx="32" cy="46" rx="24" ry="7" fill="var(--color-accent)" />
      <ellipse cx="32" cy="36" rx="22" ry="7" fill="var(--color-butter)" />
      <ellipse cx="32" cy="26" rx="20" ry="7" fill="var(--color-accent)" />
      <rect x="27" y="15" width="10" height="7" rx="2" fill="var(--color-butter)" />
    </svg>
  );
}
