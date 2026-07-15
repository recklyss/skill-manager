import type { Components } from "react-markdown";
import ReactMarkdown from "react-markdown";
import rehypeSanitize, { defaultSchema } from "rehype-sanitize";
import remarkGfm from "remark-gfm";

import { ExternalAnchor } from "./ExternalAnchor";
import "./markdown-content.css";

const sanitizeSchema = {
  ...defaultSchema,
  attributes: {
    ...defaultSchema.attributes,
    a: [...(defaultSchema.attributes?.a ?? []), "target", "rel"],
  },
};

const markdownComponents: Components = {
  a: ({ href, children, ...rest }) => {
    if (!href) {
      return <span {...rest}>{children}</span>;
    }
    return (
      <ExternalAnchor href={href} {...rest}>
        {children}
      </ExternalAnchor>
    );
  },
};

export interface MarkdownContentProps {
  markdown: string;
  className?: string;
}

export default function MarkdownContent({
  markdown,
  className = "markdown-content",
}: MarkdownContentProps) {
  return (
    <div className={className}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[[rehypeSanitize, sanitizeSchema]]}
        components={markdownComponents}
      >
        {markdown}
      </ReactMarkdown>
    </div>
  );
}
