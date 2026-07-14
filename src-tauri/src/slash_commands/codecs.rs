use super::store::SlashCommand;

pub fn render_slash_command(command: &SlashCommand, render_format: &str) -> String {
    match render_format {
        "cursor_plaintext" => PlainMarkdownCommandCodec.render(command),
        _ => FrontmatterMarkdownCommandCodec.render(command),
    }
}

pub fn parse_slash_command_document(
    name: &str,
    content: &str,
    render_format: &str,
) -> Result<SlashCommand, String> {
    match render_format {
        "cursor_plaintext" => PlainMarkdownCommandCodec.parse(name, content),
        _ => FrontmatterMarkdownCommandCodec.parse(name, content),
    }
}

struct FrontmatterMarkdownCommandCodec;

impl FrontmatterMarkdownCommandCodec {
    fn render(&self, command: &SlashCommand) -> String {
        format!(
            "---\ndescription: {}\n---\n\n{}\n",
            serde_json::to_string(command.description.trim()).unwrap_or_else(|_| "\"\"".into()),
            command.prompt.trim_end()
        )
    }

    fn parse(&self, name: &str, content: &str) -> Result<SlashCommand, String> {
        let lines: Vec<&str> = content.split('\n').collect();
        if lines.is_empty() {
            return Ok(SlashCommand {
                name: name.to_string(),
                description: name.to_string(),
                prompt: String::new(),
            });
        }
        if lines[0].trim() != "---" {
            return Ok(SlashCommand {
                name: name.to_string(),
                description: name.to_string(),
                prompt: content.trim().to_string(),
            });
        }

        let mut metadata_lines = Vec::new();
        let mut body_start = None;
        for (index, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" {
                body_start = Some(index + 1);
                break;
            }
            metadata_lines.push(*line);
        }
        let Some(body_start) = body_start else {
            return Err("invalid command frontmatter: missing closing ---".into());
        };

        let metadata = parse_frontmatter_metadata(&metadata_lines)?;
        let description = metadata
            .get("description")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .unwrap_or(name)
            .to_string();
        let prompt = lines[body_start..].join("\n").trim().to_string();
        Ok(SlashCommand {
            name: name.to_string(),
            description,
            prompt,
        })
    }
}

struct PlainMarkdownCommandCodec;

impl PlainMarkdownCommandCodec {
    fn render(&self, command: &SlashCommand) -> String {
        format!(
            "{}\n\n{}\n",
            command.description.trim_end(),
            command.prompt.trim_end()
        )
    }

    fn parse(&self, name: &str, content: &str) -> Result<SlashCommand, String> {
        let body = content.trim();
        let lines: Vec<&str> = body.lines().collect();
        let mut description = String::new();
        let mut prompt_lines = lines.as_slice();
        for (index, line) in lines.iter().enumerate() {
            if !line.trim().is_empty() {
                description = line.trim().to_string();
                prompt_lines = &lines[index + 1..];
                break;
            }
        }
        let prompt = prompt_lines.join("\n").trim().to_string();
        Ok(SlashCommand {
            name: name.to_string(),
            description: if description.is_empty() {
                name.to_string()
            } else {
                description
            },
            prompt: if prompt.is_empty() {
                body.to_string()
            } else {
                prompt
            },
        })
    }
}

fn parse_frontmatter_metadata(lines: &[&str]) -> Result<std::collections::HashMap<String, String>, String> {
    let mut metadata = std::collections::HashMap::new();
    for line in lines {
        let stripped = line.trim();
        if stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }
        let Some((key, raw_value)) = line.split_once(':') else {
            return Err(format!("invalid command frontmatter line: {line}"));
        };
        let key = key.trim();
        if key.is_empty() {
            return Err(format!("invalid command frontmatter line: {line}"));
        }
        metadata.insert(key.to_string(), parse_scalar(raw_value.trim())?);
    }
    Ok(metadata)
}

fn parse_scalar(value: &str) -> Result<String, String> {
    if value.is_empty() {
        return Ok(String::new());
    }
    if value.starts_with('"') {
        let parsed: serde_json::Value =
            serde_json::from_str(value).map_err(|_| "invalid command frontmatter string".to_string())?;
        return parsed
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| "invalid command frontmatter string".to_string());
    }
    if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
        return Ok(value[1..value.len() - 1].replace("''", "'"));
    }
    Ok(value.to_string())
}
