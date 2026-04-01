//! Configuration Screen - Edit .aether.toml

use std::path::PathBuf;
use aether_intelligence::memory::ProjectConfig;
use aether_intelligence::DubbiosoPreset;

pub struct ConfigScreen {
    /// Project root
    pub project_root: PathBuf,
    /// Current config (None if not loaded)
    pub config: Option<ProjectConfig>,
    /// Selected field index
    pub selected: usize,
    /// Is editing
    pub editing: bool,
    /// Edit buffer
    pub edit_buffer: String,
    /// Dubbioso preset selection index (0-3)
    pub dubbioso_preset_idx: usize,
}

impl ConfigScreen {
    pub fn new(project_root: PathBuf) -> Self {
        // Try to load existing config
        let config = ProjectConfig::load(&project_root).ok().flatten();

        // Get preset index from config
        let dubbioso_preset_idx = config
            .as_ref()
            .and_then(|c| c.dubbioso.preset)
            .map(preset_to_index)
            .unwrap_or(1); // Default: Balanced

        Self {
            project_root,
            config,
            selected: 0,
            editing: false,
            edit_buffer: String::new(),
            dubbioso_preset_idx,
        }
    }

    /// Load or create config
    pub fn load_or_create(&mut self) -> anyhow::Result<()> {
        if self.config.is_none() {
            self.config = Some(ProjectConfig::template());
        }
        Ok(())
    }

    /// Save config to disk
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(config) = &self.config {
            config.save(&self.project_root)?;
        }
        Ok(())
    }

    /// Apply selected Dubbioso preset to config
    pub fn apply_dubbioso_preset(&mut self) {
        if let Some(config) = &mut self.config {
            let preset = index_to_preset(self.dubbioso_preset_idx);
            config.dubbioso.preset = Some(preset);

            // Apply preset values
            let preset_config = aether_intelligence::DubbiosoConfig::from(preset);
            config.dubbioso.ask_threshold = preset_config.ask_threshold;
            config.dubbioso.warn_threshold = preset_config.warn_threshold;
            config.dubbioso.auto_accept_threshold = preset_config.auto_accept_threshold;
            config.dubbioso.permanent_after = preset_config.permanent_after;
            config.dubbioso.max_context_depth = preset_config.max_context_depth;
        }
    }

    /// Cycle to next Dubbioso preset
    pub fn next_dubbioso_preset(&mut self) {
        self.dubbioso_preset_idx = (self.dubbioso_preset_idx + 1) % 4;
        self.apply_dubbioso_preset();
    }

    /// Cycle to previous Dubbioso preset
    pub fn prev_dubbioso_preset(&mut self) {
        self.dubbioso_preset_idx = if self.dubbioso_preset_idx == 0 { 3 } else { self.dubbioso_preset_idx - 1 };
        self.apply_dubbioso_preset();
    }

    /// Available config fields
    pub fn fields(&self) -> Vec<ConfigField> {
        vec![
            ConfigField::DubbiosoPreset,
            ConfigField::Threshold("complexity.max_cyclomatic"),
            ConfigField::Threshold("function.max_lines"),
            ConfigField::Threshold("function.max_params"),
            ConfigField::Threshold("file.max_lines"),
            ConfigField::Whitelist,
            ConfigField::Style,
            ConfigField::CustomRules,
        ]
    }

    /// Get current Dubbioso preset name
    pub fn dubbioso_preset_name(&self) -> &'static str {
        preset_name(self.dubbioso_preset_idx)
    }

    /// Get current Dubbioso preset description
    pub fn dubbioso_preset_desc(&self) -> &'static str {
        preset_description(self.dubbioso_preset_idx)
    }
}

pub enum ConfigField {
    DubbiosoPreset,
    Threshold(&'static str),
    Whitelist,
    Style,
    CustomRules,
}

impl ConfigField {
    pub fn label(&self) -> &str {
        match self {
            ConfigField::DubbiosoPreset => "Dubbioso Mode",
            ConfigField::Threshold(name) => name,
            ConfigField::Whitelist => "Whitelist",
            ConfigField::Style => "Style Conventions",
            ConfigField::CustomRules => "Custom Rules",
        }
    }
}

/// Convert preset enum to index
fn preset_to_index(preset: DubbiosoPreset) -> usize {
    match preset {
        DubbiosoPreset::Strict => 0,
        DubbiosoPreset::Balanced => 1,
        DubbiosoPreset::Fast => 2,
        DubbiosoPreset::Turbo => 3,
    }
}

/// Convert index to preset enum
fn index_to_preset(idx: usize) -> DubbiosoPreset {
    match idx {
        0 => DubbiosoPreset::Strict,
        1 => DubbiosoPreset::Balanced,
        2 => DubbiosoPreset::Fast,
        3 => DubbiosoPreset::Turbo,
        _ => DubbiosoPreset::Balanced,
    }
}

/// Get preset name by index
fn preset_name(idx: usize) -> &'static str {
    match idx {
        0 => "Strict",
        1 => "Balanced",
        2 => "Fast",
        3 => "Turbo",
        _ => "Balanced",
    }
}

/// Get preset description by index
fn preset_description(idx: usize) -> &'static str {
    match idx {
        0 => "Max precision, asks often, slow learning",
        1 => "Balanced precision/speed (default)",
        2 => "Fast, fewer questions, quick learning",
        3 => "Max speed, no questions, rapid learning",
        _ => "Balanced precision/speed (default)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_cycling_next() {
        let mut screen = ConfigScreen::new(std::path::PathBuf::from("/tmp"));
        screen.config = Some(ProjectConfig::template());

        // Start at Balanced (index 1)
        assert_eq!(screen.dubbioso_preset_idx, 1);

        screen.next_dubbioso_preset();
        assert_eq!(screen.dubbioso_preset_idx, 2);

        screen.next_dubbioso_preset();
        assert_eq!(screen.dubbioso_preset_idx, 3);

        // Wrap around
        screen.next_dubbioso_preset();
        assert_eq!(screen.dubbioso_preset_idx, 0);
    }

    #[test]
    fn test_preset_cycling_prev() {
        let mut screen = ConfigScreen::new(std::path::PathBuf::from("/tmp"));
        screen.config = Some(ProjectConfig::template());
        screen.dubbioso_preset_idx = 0;

        // Wrap around backwards
        screen.prev_dubbioso_preset();
        assert_eq!(screen.dubbioso_preset_idx, 3);

        screen.prev_dubbioso_preset();
        assert_eq!(screen.dubbioso_preset_idx, 2);
    }

    #[test]
    fn test_preset_name_matches_index() {
        assert_eq!(preset_name(0), "Strict");
        assert_eq!(preset_name(1), "Balanced");
        assert_eq!(preset_name(2), "Fast");
        assert_eq!(preset_name(3), "Turbo");
    }

    #[test]
    fn test_index_to_preset_roundtrip() {
        for i in 0..4 {
            let preset = index_to_preset(i);
            assert_eq!(preset_to_index(preset), i);
        }
    }
}
