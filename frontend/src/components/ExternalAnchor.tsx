import type { AnchorHTMLAttributes, MouseEvent, ReactNode } from "react";

import { isTauriRuntime, openExternalUrl } from "../lib/openExternalUrl";

interface ExternalAnchorProps extends Omit<AnchorHTMLAttributes<HTMLAnchorElement>, "href"> {
  href: string;
  children: ReactNode;
}

export function ExternalAnchor({
  href,
  children,
  onClick,
  target = "_blank",
  rel = "noopener noreferrer",
  ...rest
}: ExternalAnchorProps) {
  function handleClick(event: MouseEvent<HTMLAnchorElement>): void {
    onClick?.(event);
    if (event.defaultPrevented) {
      return;
    }

    if (!isTauriRuntime()) {
      return;
    }

    event.preventDefault();
    void openExternalUrl(href);
  }

  return (
    <a href={href} target={target} rel={rel} onClick={handleClick} {...rest}>
      {children}
    </a>
  );
}
