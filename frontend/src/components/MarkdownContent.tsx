import type { Components } from "react-markdown";
import ReactMarkdown from "react-markdown";
import rehypeSanitize, { defaultSchema } from "rehype-sanitize";
import remarkGfm from "remark-gfm";

import "./markdown-content.css";

const sanitizeSchema = {
  ...defaultSchema,
  attributes: {
    ...defaultSchema.attributes,
    a: [...(defaultSchema.attributes?.a ?? []), "target", "rel"],
  },
};

const markdownComponents: Components = {
  a: ({ href, children, ...rest }) => (
    <a href={href} target="_blank" rel="noopener noreferrer" {...rest}>
      {children}
    </a>
  ),
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
