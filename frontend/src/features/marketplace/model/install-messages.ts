const ALREADY_INSTALLED_PATTERNS = [
  "already installed",
  "already exists",
  "file exists",
  "package directory already exists",
];

export function friendlyMarketplaceInstallError(message: string): string {
  const lower = message.toLowerCase();
  if (ALREADY_INSTALLED_PATTERNS.some((pattern) => lower.includes(pattern))) {
    return "Already installed. Choose Re-install to update.";
  }
  return message;
}
