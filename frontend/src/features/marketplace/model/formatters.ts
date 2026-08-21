/** Strip basic markdown formatting for a plain-text card preview. */
export function stripMarkdownForPreview(markdown: string): string {
  return markdown
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")  // links → text
    .replace(/[*_~`]{1,3}/g, "")               // bold / italic / strikethrough / inline code
    .replace(/^#{1,6}\s+/gm, "")               // heading markers
    .replace(/^[-*+]\s+/gm, "")                 // unordered list markers
    .replace(/^\d+\.\s+/gm, "")                 // ordered list markers
    .replace(/^>\s*/gm, "")                     // blockquote markers
    .replace(/\n{2,}/g, " ")                    // paragraph breaks → space
    .replace(/\s+/g, " ")                       // collapse whitespace
    .trim();
}

export function formatMarketplaceStars(value: number): string {
  if (value >= 1000) {
    return `${(value / 1000).toFixed(1)}k`;
  }
  return `${value}`;
}

export function formatMarketplaceInstalls(value: number): string {
  if (value >= 1000000) {
    return `${(value / 1000000).toFixed(1)}M`;
  }
  if (value >= 1000) {
    return `${(value / 1000).toFixed(1)}K`;
  }
  return `${value}`;
}

export function formatMcpUseCount(value: number): string {
  if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(1)}M`;
  }
  if (value >= 1000) {
    return `${(value / 1000).toFixed(1)}k`;
  }
  return `${value}`;
}
