/**
 * The Pancake Compressor mark: a stack of three pancakes with a wider,
 * flatter middle layer (the "squash"), matching `assets/brand/logo-master.svg`
 * and the app icon. Decorative here, so hidden from assistive technology;
 * the product name is always next to it in text.
 */
export function PancakeMark({ className = "" }: { className?: string }) {
  return (
    <svg className={className} viewBox="40 120 432 280" aria-hidden="true" focusable="false">
      <rect x="92" y="300" width="328" height="78" rx="39" fill="#8A4A1A" />
      <rect x="92" y="300" width="328" height="58" rx="29" fill="#D98B3A" />
      <rect x="52" y="244" width="408" height="58" rx="29" fill="#8A4A1A" />
      <rect x="52" y="244" width="408" height="40" rx="20" fill="#E9A54B" />
      <rect x="104" y="170" width="304" height="76" rx="38" fill="#8A4A1A" />
      <rect x="104" y="170" width="304" height="56" rx="28" fill="#F0B25A" />
      <rect x="216" y="140" width="80" height="44" rx="12" fill="#F8D27D" stroke="#C9963D" strokeWidth="6" />
    </svg>
  );
}
