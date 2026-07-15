/**
 * WCAG 2.2 contrast utilities (relative luminance + contrast ratio).
 * https://www.w3.org/TR/WCAG22/#contrast-minimum
 */

export const WCAG_AAA_NORMAL = 7;
export const WCAG_AAA_LARGE = 4.5;
export const WCAG_AAA_UI = 3;

function hexToRgb(hex: string): [number, number, number] {
  const normalized = hex.replace("#", "").trim();
  if (normalized.length === 3) {
    const r = normalized[0] + normalized[0];
    const g = normalized[1] + normalized[1];
    const b = normalized[2] + normalized[2];
    return [
      Number.parseInt(r, 16),
      Number.parseInt(g, 16),
      Number.parseInt(b, 16),
    ];
  }
  return [
    Number.parseInt(normalized.slice(0, 2), 16),
    Number.parseInt(normalized.slice(2, 4), 16),
    Number.parseInt(normalized.slice(4, 6), 16),
  ];
}

function srgbChannelToLinear(channel: number): number {
  const s = channel / 255;
  return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
}

/** Relative luminance for sRGB hex colors (opaque). */
export function relativeLuminance(hex: string): number {
  const [r, g, b] = hexToRgb(hex);
  const rl = srgbChannelToLinear(r);
  const gl = srgbChannelToLinear(g);
  const bl = srgbChannelToLinear(b);
  return 0.2126 * rl + 0.7152 * gl + 0.0722 * bl;
}

/** Contrast ratio between two opaque sRGB hex colors (always >= 1). */
export function contrastRatio(foreground: string, background: string): number {
  const l1 = relativeLuminance(foreground);
  const l2 = relativeLuminance(background);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

export function meetsContrast(
  foreground: string,
  background: string,
  minimum: number,
): boolean {
  return contrastRatio(foreground, background) >= minimum;
}

/**
 * Nudge `foreground` toward higher contrast against `background` until `target` is met.
 * Preserves hue direction: darkens foreground on light backgrounds and lightens on dark.
 */
export function nudgeForContrast(
  foreground: string,
  background: string,
  target: number,
): string {
  let [r, g, b] = foreground
    .replace("#", "")
    .match(/.{2}/g)!
    .map((hex) => Number.parseInt(hex, 16));
  const increaseSeparation =
    relativeLuminance(foreground) < relativeLuminance(background);

  for (let step = 0; step < 128; step += 1) {
    const hex =
      "#" +
      [r, g, b].map((channel) => channel.toString(16).padStart(2, "0")).join("");
    if (contrastRatio(hex, background) >= target) {
      return hex;
    }
    if (increaseSeparation) {
      r = Math.max(0, r - 2);
      g = Math.max(0, g - 2);
      b = Math.max(0, b - 2);
    } else {
      r = Math.min(255, r + 2);
      g = Math.min(255, g + 2);
      b = Math.min(255, b + 2);
    }
  }

  return (
    "#" + [r, g, b].map((channel) => channel.toString(16).padStart(2, "0")).join("")
  );
}

/** Ensure `color` meets `target` contrast on every surface in `surfaces`. */
export function adjustForSurfaces(
  color: string,
  surfaces: string[],
  target: number,
): string {
  let result = color;
  for (const surface of surfaces) {
    if (!surface.startsWith("#")) continue;
    if (contrastRatio(result, surface) < target) {
      result = nudgeForContrast(result, surface, target);
    }
  }
  return result;
}
