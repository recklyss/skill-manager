use regex::Regex;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub prompt: String,
}

#[derive(Clone)]
pub struct SlashCommandStore {
    commands_dir: PathBuf,
}

impl SlashCommandStore {
    pub fn new(commands_dir: PathBuf) -> Self {
        Self { commands_dir }
    }

    pub fn list_commands(&self) -> Vec<SlashCommand> {
        let mut commands = Vec::new();
        let Ok(entries) = fs::read_dir(&self.commands_dir) else {
            return commands;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }
            if let Ok(command) = self.read_command_path(&path) {
                commands.push(command);
            }
        }
        commands.sort_by(|a, b| a.name.cmp(&b.name));
        commands
    }

    pub fn get_command(&self, name: &str) -> Option<SlashCommand> {
        validate_command_name(name).ok()?;
        let path = self.command_path(name);
        if !path.is_file() {
            return None;
        }
        self.read_command_path(&path).ok()
    }

    pub fn create_command(&self, command: &SlashCommand) -> Result<SlashCommand, String> {
        validate_command(command)?;
        let path = self.command_path(&command.name);
        if path.exists() {
            return Err(format!("slash command already exists: {}", command.name));
        }
        self.write_command_path(&path, command)?;
        Ok(command.clone())
    }

    pub fn update_command(&self, name: &str, description: &str, prompt: &str) -> Result<SlashCommand, String> {
        validate_command_name(name)?;
        let path = self.command_path(name);
        if !path.is_file() {
            return Err(format!("unknown slash command: {name}"));
        }
        let command = SlashCommand {
            name: name.to_string(),
            description: description.to_string(),
            prompt: prompt.to_string(),
        };
        validate_command(&command)?;
        self.write_command_path(&path, &command)?;
        Ok(command)
    }

    pub fn delete_command(&self, name: &str) -> Result<(), String> {
        validate_command_name(name)?;
        let path = self.command_path(name);
        if !path.is_file() {
            return Err(format!("unknown slash command: {name}"));
        }
        fs::remove_file(path).map_err(|e| e.to_string())
    }

    fn command_path(&self, name: &str) -> PathBuf {
        self.commands_dir.join(format!("{name}.toml"))
    }

    fn read_command_path(&self, path: &PathBuf) -> Result<SlashCommand, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let payload: toml::Value = toml::from_str(&text).map_err(|e| format!("invalid command TOML: {e}"))?;
        let command = SlashCommand {
            name: string_field(&payload, "name"),
            description: string_field(&payload, "description"),
            prompt: string_field(&payload, "prompt"),
        };
        validate_command(&command)?;
        if path.file_stem().and_then(|s| s.to_str()) != Some(command.name.as_str()) {
            return Err(format!(
                "command name must match filename '{}'",
                path.file_stem().and_then(|s| s.to_str()).unwrap_or("")
            ));
        }
        Ok(command)
    }

    fn write_command_path(&self, path: &PathBuf, command: &SlashCommand) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let payload = format!(
            "name = {:?}\ndescription = {:?}\nprompt = {:?}\n",
            command.name,
            command.description.trim(),
            command.prompt.trim_end_matches('\n')
        );
        let temp = path.with_extension("toml.tmp");
        fs::write(&temp, payload).map_err(|e| e.to_string())?;
        fs::rename(&temp, path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn string_field(payload: &toml::Value, key: &str) -> String {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

pub fn validate_command_name(name: &str) -> Result<(), String> {
    let re = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    if !re.is_match(name) {
        return Err("name must use lowercase letters, numbers, and hyphens".into());
    }
    Ok(())
}

pub fn validate_command(command: &SlashCommand) -> Result<(), String> {
    validate_command_name(&command.name)?;
    if command.description.trim().is_empty() {
        return Err("description is required".into());
    }
    if command.prompt.trim().is_empty() {
        return Err("prompt is required".into());
    }
    Ok(())
}
