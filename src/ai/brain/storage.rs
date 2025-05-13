use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use crate::ai::brain::Brain;
use crate::ai::config::AIConfig;

// Handles saving and loading of brain and configuration
pub struct Storage {
    brain_path: String,
    #[allow(dead_code)]
    config_path: String,
    #[allow(dead_code)]
    history_path: String,
    #[allow(dead_code)]
    autosave_interval: u64, // In seconds
}

impl Storage {
    pub fn new(brain_path: &str, config_path: &str, history_path: &str, autosave_interval: u64) -> Self {
        Self {
            brain_path: brain_path.to_string(),
            config_path: config_path.to_string(),
            history_path: history_path.to_string(),
            autosave_interval,
        }
    }
    
    // Save the brain to a file
    #[allow(dead_code)]
    pub fn save_brain(&self, brain: &Brain) -> Result<(), String> {
        // Create backup of existing file if it exists
        if Path::new(&self.brain_path).exists() {
            let backup_path = format!("{}.bak", self.brain_path);
            fs::copy(&self.brain_path, &backup_path)
                .map_err(|e| format!("Failed to create backup: {}", e))?;
        }
        
        // Serialize and save
        let serialized = serde_json::to_string(brain)
            .map_err(|e| format!("Failed to serialize brain: {}", e))?;
        
        let mut file = File::create(&self.brain_path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        
        file.write_all(serialized.as_bytes())
            .map_err(|e| format!("Failed to write to file: {}", e))?;
        
        Ok(())
    }
    
    // Load a brain from a file
    #[allow(dead_code)]
    pub fn load_brain(&self) -> Result<Brain, String> {
        let mut file = File::open(&self.brain_path)
            .map_err(|e| format!("Failed to open brain file: {}", e))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read brain file: {}", e))?;
        
        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to deserialize brain: {}", e))
    }
    
    // Check if a saved brain exists
    #[allow(dead_code)]
    pub fn brain_exists(&self) -> bool {
        Path::new(&self.brain_path).exists()
    }
    
    // Load configuration from YAML
    #[allow(dead_code)]
    pub fn load_config(&self) -> Result<AIConfig, String> {
        AIConfig::load_from_yaml(&self.config_path)
    }
    
    // Save configuration to YAML
    #[allow(dead_code)]
    pub fn save_config(&self, config: &AIConfig) -> Result<(), String> {
        let serialized = serde_yaml::to_string(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        let mut file = File::create(&self.config_path)
            .map_err(|e| format!("Failed to create config file: {}", e))?;
        
        file.write_all(serialized.as_bytes())
            .map_err(|e| format!("Failed to write to config file: {}", e))?;
        
        Ok(())
    }
}
