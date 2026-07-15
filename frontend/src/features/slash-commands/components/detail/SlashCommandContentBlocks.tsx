import MarkdownContent from "../../../../components/MarkdownContent";
import { DetailSection } from "../../../../components/detail/DetailSection";
import type { SlashRenderFormat } from "../../api/types";
import { useSlashCommandsCopy } from "../../i18n";

interface SlashCommandTextProps {
  description?: string | null;
  prompt?: string | null;
  descriptionEmptyText?: string;
  promptEmptyText?: string;
  promptRenderFormat?: SlashRenderFormat;
}

export function SlashCommandContentSections({
  description,
  prompt,
  descriptionEmptyText,
  promptEmptyText,
  promptRenderFormat = "frontmatter_markdown",
}: SlashCommandTextProps) {
  const copy = useSlashCommandsCopy();

  return (
    <>
      <DetailSection heading={copy.detail.description}>
        <SlashCommandDescriptionBlock
          description={description}
          emptyText={descriptionEmptyText}
        />
      </DetailSection>
      <DetailSection heading={copy.detail.prompt}>
        <SlashCommandPromptPreview
          prompt={prompt}
          emptyText={promptEmptyText}
          renderFormat={promptRenderFormat}
        />
      </DetailSection>
    </>
  );
}

export function SlashCommandSourcePreview({
  description,
  prompt,
  descriptionEmptyText,
  promptEmptyText,
  promptRenderFormat = "frontmatter_markdown",
}: SlashCommandTextProps) {
  const copy = useSlashCommandsCopy();

  return (
    <div className="slash-command-detail__content-preview">
      <div className="slash-command-detail__content-field">
        <span>{copy.detail.description}</span>
        <SlashCommandDescriptionBlock
          description={description}
          emptyText={descriptionEmptyText}
        />
      </div>
      <div className="slash-command-detail__content-field">
        <span>{copy.detail.prompt}</span>
        <SlashCommandPromptPreview
          prompt={prompt}
          emptyText={promptEmptyText}
          renderFormat={promptRenderFormat}
        />
      </div>
    </div>
  );
}

export function SlashCommandDescriptionBlock({
  description,
  emptyText,
}: {
  description?: string | null;
  emptyText?: string;
}) {
  const copy = useSlashCommandsCopy();

  return (
    <div className="slash-command-detail__description-block">
      <p className="slash-command-detail__description-text">
        {description?.trim() || emptyText || copy.detail.noDescription}
      </p>
    </div>
  );
}

export function SlashCommandPromptPreview({
  prompt,
  emptyText,
  renderFormat = "frontmatter_markdown",
}: {
  prompt?: string | null;
  emptyText?: string;
  renderFormat?: SlashRenderFormat;
}) {
  const copy = useSlashCommandsCopy();
  const content = prompt?.trim() || emptyText || copy.detail.noPrompt;

  if (renderFormat === "cursor_plaintext") {
    return <pre className="slash-command-detail__prompt ui-scrollbar">{content}</pre>;
  }

  return (
    <MarkdownContent
      markdown={content}
      className="markdown-content slash-command-detail__prompt-markdown"
    />
  );
}
