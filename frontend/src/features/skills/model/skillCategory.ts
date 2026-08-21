import type { SkillListRow } from "./types";

export type SkillCategoryId =
  | "coding"
  | "docs"
  | "media"
  | "memory"
  | "research"
  | "devops"
  | "security"
  | "agents"
  | "planning"
  | "data"
  | "communication"
  | "other";

export const CATEGORY_ORDER: SkillCategoryId[] = [
  "coding",
  "docs",
  "media",
  "memory",
  "research",
  "agents",
  "devops",
  "security",
  "planning",
  "data",
  "communication",
  "other",
];

type SlugRule = {
  id: SkillCategoryId;
  exact?: readonly string[];
  prefix?: readonly string[];
  slugPattern?: RegExp;
  token?: readonly string[];
};

/** Classify from the skill slug/name first — avoids description false positives like "topic/notes". */
const SLUG_RULES: readonly SlugRule[] = [
  {
    id: "memory",
    exact: ["apple-notes", "memory-manager", "memory-merger", "learn"],
    prefix: ["memory-"],
  },
  {
    id: "media",
    exact: [
      "frontend-slides",
      "slidev",
      "pptx-generator",
      "gemini-image-generator",
      "image-to-video",
      "video-editing",
      "threejs-builder",
      "nano-banana-builder",
      "distinctive-frontend-design",
      "create-voice",
      "voice-orchestrator",
      "voice-validator",
      "wordpress-live-validation",
      "wordpress-uploader",
    ],
    prefix: ["image-", "video-", "voice-"],
    slugPattern: /(?:^|[-_])(slides?|pptx|figma|canvas|thumbnail|screenshot)(?:$|[-_])/,
    token: ["slides", "slide", "pptx", "figma", "canvas", "thumbnail", "screenshot", "wordpress"],
  },
  {
    id: "research",
    prefix: ["firecrawl"],
    exact: [
      "bluesky-reader",
      "reddit-moderate",
      "find-skills",
      "link-auditor",
      "codebase-overview",
      "read-only-ops",
      "service-health-check",
    ],
    token: ["firecrawl", "bluesky", "reddit"],
  },
  {
    id: "security",
    exact: [
      "security-threat-model",
      "sapcc-audit",
      "sapcc-review",
      "kubernetes-security",
      "cron-job-auditor",
      "image-auditor",
      "go-sapcc-conventions",
      "repo-value-analysis",
    ],
    prefix: ["sapcc-"],
    token: ["threat", "vulnerab", "sanitize"],
  },
  {
    id: "agents",
    exact: [
      "subagent-driven-development",
      "dispatching-parallel-agents",
      "skill-composer",
      "skill-eval",
      "testing-agents-with-subagents",
      "self-improving-agent",
      "routing-table-updater",
      "agent-comparison",
      "agent-eval",
      "agent-evaluation",
    ],
    prefix: ["agent-"],
    token: ["subagent", "dispatch", "orchestrat", "babysit"],
  },
  {
    id: "devops",
    exact: [
      "github-actions-check",
      "kubernetes-debugging",
      "headless-cron-creator",
      "feature-release",
      "perses-deploy",
      "pr-sync",
      "pr-status",
      "pr-cleanup",
      "condition-based-waiting",
      "install",
      "fish-shell-config",
    ],
    prefix: ["kubernetes-", "github-actions"],
    token: ["deploy", "cron", "homebrew", "k8s", "docker", "vercel"],
  },
  {
    id: "data",
    prefix: ["perses-"],
    exact: ["data-analysis"],
    token: ["grafana", "metrics", "sql", "analytics"],
  },
  {
    id: "communication",
    exact: [
      "teach",
      "grill-me",
      "roast",
      "socratic-debugging",
      "ask-matt",
      "professional-communication",
      "pair-programming",
      "receiving-code-review",
      "requesting-code-review",
    ],
    token: ["teach", "grill", "roast", "socratic"],
  },
  {
    id: "planning",
    exact: [
      "brainstorming",
      "feature-plan",
      "feature-design",
      "feature-implement",
      "pre-planning-discussion",
      "decision-helper",
      "plan-manager",
      "plan-checker",
      "planning-with-files",
      "plans",
      "plant-seed",
      "content-calendar",
      "series-planner",
      "workflow-help",
      "resume-work",
      "pause-work",
      "retro",
      "topic-brainstormer",
      "pre-publish-checker",
      "content-engine",
    ],
    prefix: ["feature-plan", "feature-design", "feature-implement", "plan-"],
    token: ["brainstorm", "roadmap", "calendar", "workflow", "series"],
  },
  {
    id: "docs",
    exact: [
      "blog-post-writer",
      "writing-skills",
      "batch-editor",
      "post-outliner",
      "generate-claudemd",
      "spec-writer",
      "seo-optimizer",
      "taxonomy-manager",
      "comment-quality",
      "anti-ai-editor",
      "adr-consultation",
      "vercel-react-best-practices",
      "with-anti-rationalization",
    ],
    token: ["blog", "writing", "claudemd", "readme", "markdown", "outline", "changelog", "adr"],
  },
  {
    id: "coding",
    prefix: [
      "go-",
      "kotlin-",
      "swift-",
      "php-",
      "code-",
      "feature-validate",
      "systematic-",
      "test-",
      "testing-",
      "typescript-",
      "parallel-code-",
      "branch-",
      "git-",
      "using-git-",
      "finishing-",
      "full-repo-",
      "codebase-analyzer",
      "integration-",
      "pr-miner",
      "pr-mining",
      "pr-review",
      "pr-fix",
      "python-quality",
      "universal-quality",
      "endpoint-validator",
      "github-notification",
    ],
    exact: [
      "code-cleanup",
      "code-linting",
      "codebase-analyzer",
      "feature-validate",
      "parallel-code-review",
      "typescript-check",
      "universal-quality-gate",
      "using-git-worktrees",
      "finishing-a-development-branch",
      "full-repo-review",
      "integration-checker",
      "pr-miner",
      "pr-mining-coordinator",
      "pr-review-address-feedback",
      "pr-fix",
      "python-quality-gate",
      "test-driven-development",
      "testing-anti-patterns",
      "systematic-debugging",
      "systematic-refactoring",
      "systematic-code-review",
      "github-notification-triage",
      "joy-check",
      "quick",
      "fast",
      "ponytail",
    ],
    slugPattern:
      /(?:^|[-_])(code|coding|refactor|debug|lint|review|typescript|python|rust|kotlin|swift|php|implement|cleanup|endpoint|api|frontend|backend|commit|branch|quality-gate)(?:$|[-_])/,
    token: [
      "code",
      "coding",
      "refactor",
      "debug",
      "lint",
      "review",
      "typescript",
      "python",
      "rust",
      "kotlin",
      "swift",
      "php",
      "implement",
      "cleanup",
      "endpoint",
      "commit",
      "branch",
    ],
  },
];

/** Conservative description-only hints when the slug is ambiguous. */
const DESCRIPTION_RULES: readonly { id: SkillCategoryId; pattern: RegExp }[] = [
  { id: "memory", pattern: /\b(long[- ]term memory|knowledge base|note[- ]taking|remember across sessions)\b/i },
  { id: "media", pattern: /\b(html presentation|image generation|video generation|audio generation|slide deck)\b/i },
  { id: "research", pattern: /\b(web search|web scrape|market research|literature review)\b/i },
  { id: "security", pattern: /\b(security audit|threat model|vulnerability)\b/i },
  { id: "agents", pattern: /\b(subagent|multi[- ]agent|agent harness)\b/i },
  { id: "devops", pattern: /\b(ci\/cd|continuous integration|kubernetes cluster|deploy to production)\b/i },
  { id: "planning", pattern: /\b(design phase|implementation plan|roadmap planning)\b/i },
  { id: "docs", pattern: /\b(blog post|technical writing|documentation generation)\b/i },
  { id: "data", pattern: /\b(data analysis|sql query|metrics dashboard)\b/i },
  { id: "communication", pattern: /\b(code review feedback|teaching|socratic)\b/i },
  { id: "coding", pattern: /\b(refactor|unit test|typecheck|pull request)\b/i },
];

function skillSlug(row: SkillListRow): string {
  const fromRef = row.skillRef.includes(":") ? row.skillRef.split(":").pop() : row.skillRef;
  return (fromRef ?? row.name).toLowerCase();
}

function slugTokens(slug: string): string[] {
  return slug.split(/[-_/]+/).filter(Boolean);
}

function matchesSlugRule(slug: string, tokens: string[], rule: SlugRule): boolean {
  if (rule.exact?.includes(slug)) {
    return true;
  }
  if (rule.prefix?.some((prefix) => slug === prefix || slug.startsWith(prefix))) {
    return true;
  }
  if (rule.slugPattern?.test(slug)) {
    return true;
  }
  if (rule.token?.some((needle) => tokens.includes(needle) || slug.includes(needle))) {
    return true;
  }
  return false;
}

function classifyFromSlug(slug: string): SkillCategoryId | null {
  const tokens = slugTokens(slug);
  // Exact slugs win over prefix/token rules (e.g. image-auditor → security, not media's image- prefix).
  for (const rule of SLUG_RULES) {
    if (rule.exact?.includes(slug)) {
      return rule.id;
    }
  }
  for (const rule of SLUG_RULES) {
    if (matchesSlugRule(slug, tokens, rule)) {
      return rule.id;
    }
  }
  return null;
}

function classifyFromDescription(description: string): SkillCategoryId | null {
  const haystack = description.toLowerCase();
  for (const rule of DESCRIPTION_RULES) {
    if (rule.pattern.test(haystack)) {
      return rule.id;
    }
  }
  return null;
}

export function classifySkillCategory(row: SkillListRow): SkillCategoryId {
  const slug = skillSlug(row);
  return classifyFromSlug(slug) ?? classifyFromDescription(row.description) ?? "other";
}

export interface GroupedSkillRows {
  category: SkillCategoryId;
  rows: SkillListRow[];
}

export function groupRowsByCategory(rows: SkillListRow[]): GroupedSkillRows[] {
  const buckets = new Map<SkillCategoryId, SkillListRow[]>();
  for (const row of rows) {
    const category = classifySkillCategory(row);
    const list = buckets.get(category) ?? [];
    list.push(row);
    buckets.set(category, list);
  }
  return CATEGORY_ORDER.filter((category) => (buckets.get(category)?.length ?? 0) > 0).map((category) => ({
    category,
    rows: buckets.get(category)!,
  }));
}
