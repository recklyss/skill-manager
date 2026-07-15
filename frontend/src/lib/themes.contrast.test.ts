import { describe, expect, it } from "vitest";

import {
  contrastRatio,
  WCAG_AAA_LARGE,
  WCAG_AAA_NORMAL,
  WCAG_AAA_UI,
} from "./contrast";
import { THEMES } from "./themes";

const TEXT_BACKGROUNDS = [
  "--color-bg",
  "--color-surface",
  "--color-surface-raised",
] as const;

const BORDER_ADJACENT = [
  "--color-bg",
  "--color-surface",
  "--color-surface-raised",
] as const;

const SEMANTIC_COLORS = [
  "--color-success",
  "--color-danger",
  "--color-warning",
] as const;

describe("theme contrast (WCAG 2.2 AAA)", () => {
  for (const theme of THEMES) {
    describe(theme.id, () => {
      for (const bgKey of TEXT_BACKGROUNDS) {
        const bg = theme.tokens[bgKey];
        if (!bg?.startsWith("#")) continue;

        it(`--color-text on ${bgKey} meets AAA normal (7:1)`, () => {
          const ratio = contrastRatio(theme.tokens["--color-text"], bg);
          expect(
            ratio,
            `${theme.id} text on ${bgKey}: ${ratio.toFixed(2)}:1`,
          ).toBeGreaterThanOrEqual(WCAG_AAA_NORMAL);
        });

        it(`--color-text-muted on ${bgKey} meets AAA normal (7:1)`, () => {
          const ratio = contrastRatio(theme.tokens["--color-text-muted"], bg);
          expect(
            ratio,
            `${theme.id} text-muted on ${bgKey}: ${ratio.toFixed(2)}:1`,
          ).toBeGreaterThanOrEqual(WCAG_AAA_NORMAL);
        });
      }

      for (const bgKey of BORDER_ADJACENT) {
        const bg = theme.tokens[bgKey];
        if (!bg?.startsWith("#")) continue;

        it(`--color-border on ${bgKey} meets AAA UI (3:1)`, () => {
          const ratio = contrastRatio(theme.tokens["--color-border"], bg);
          expect(
            ratio,
            `${theme.id} border on ${bgKey}: ${ratio.toFixed(2)}:1`,
          ).toBeGreaterThanOrEqual(WCAG_AAA_UI);
        });
      }

      for (const bgKey of TEXT_BACKGROUNDS) {
        const bg = theme.tokens[bgKey];
        if (!bg?.startsWith("#")) continue;

        it(`--color-accent on ${bgKey} meets AAA large text (4.5:1)`, () => {
          const ratio = contrastRatio(theme.tokens["--color-accent"], bg);
          expect(
            ratio,
            `${theme.id} accent on ${bgKey}: ${ratio.toFixed(2)}:1`,
          ).toBeGreaterThanOrEqual(WCAG_AAA_LARGE);
        });
      }

      const accent = theme.tokens["--color-accent"];
      const inverted = theme.tokens["--color-text-inverted"];
      if (accent?.startsWith("#") && inverted?.startsWith("#")) {
        it("--color-text-inverted on --color-accent meets AAA normal (7:1)", () => {
          const ratio = contrastRatio(inverted, accent);
          expect(
            ratio,
            `${theme.id} inverted on accent: ${ratio.toFixed(2)}:1`,
          ).toBeGreaterThanOrEqual(WCAG_AAA_NORMAL);
        });
      }

      for (const semanticKey of SEMANTIC_COLORS) {
        const color = theme.tokens[semanticKey];
        if (!color?.startsWith("#")) continue;

        for (const bgKey of ["--color-surface", "--color-surface-raised"] as const) {
          const bg = theme.tokens[bgKey];
          if (!bg?.startsWith("#")) continue;

          it(`${semanticKey} on ${bgKey} meets AAA large text (4.5:1)`, () => {
            const ratio = contrastRatio(color, bg);
            expect(
              ratio,
              `${theme.id} ${semanticKey} on ${bgKey}: ${ratio.toFixed(2)}:1`,
            ).toBeGreaterThanOrEqual(WCAG_AAA_LARGE);
          });
        }
      }
    });
  }
});
