// Shared envelope mechanics (N-023). Pure, DOM-free, framework-agnostic —
// the substrate both `core` components and app shells build on. ≈ xgen-common (D-095).

/**
 * Derive a component's kebab-case type-class from its name (N-020).
 * `OutboxCard` -> `outbox-card`; an already-kebab name is returned unchanged.
 */
export function kebab(name: string): string {
  return name
    .replace(/([a-z0-9])([A-Z])/g, '$1-$2')
    .replace(/[\s_]+/g, '-')
    .toLowerCase();
}

/**
 * Merge the component's own type-class with caller-supplied classes.
 * Supplies the type-class, never erases it (the N-023 guarantee).
 */
export function mergeClasses(typeClass: string, passthrough?: string | null): string {
  const extra = (passthrough ?? '').trim();
  return extra ? `${typeClass} ${extra}` : typeClass;
}
