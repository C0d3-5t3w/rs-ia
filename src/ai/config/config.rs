use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub input_size: usize,
    pub hidden_size1: usize,
    pub hidden_size2: usize,
    pub output_size: usize,
    pub learning_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub discount_factor: f64,
    pub exploration_rate: f64,
    pub exploration_decay: f64,
    pub min_exploration_rate: f64,
    pub batch_size: usize,
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub grid_size: usize,
    pub maze_width: usize, 
    pub maze_height: usize,
    pub wall_thickness: usize,
    pub movement_speed: usize,
    pub max_frames_per_game: u32,
    #[serde(default = "default_gravity")]
    pub gravity: f64,
    #[serde(default = "default_jump_velocity")]
    pub jump_velocity: f64,
    #[serde(default = "default_bird_size")]
    pub bird_size: f64,
    #[serde(default = "default_pipe_width")]
    pub pipe_width: f64,
    #[serde(default = "default_pipe_gap")]
    pub pipe_gap: f64,
    #[serde(default = "default_pipe_spawn_distance")]
    pub pipe_spawn_distance: f64,
}

fn default_gravity() -> f64 { 0.0 }
fn default_jump_velocity() -> f64 { 0.0 }
fn default_bird_size() -> f64 { 0.0 }
fn default_pipe_width() -> f64 { 0.0 }
fn default_pipe_gap() -> f64 { 0.0 }
fn default_pipe_spawn_distance() -> f64 { 0.0 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub network: NetworkConfig,
    pub training: TrainingConfig,
    pub environment: EnvironmentConfig,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                input_size: 8,
                hidden_size1: 32,
                hidden_size2: 16,
                output_size: 1,
                learning_rate: 0.005,
            },
            training: TrainingConfig {
                discount_factor: 0.99,
                exploration_rate: 1.0,
                exploration_decay: 0.997,
                min_exploration_rate: 0.01,
                batch_size: 64,
                buffer_size: 20000,
            },
            environment: EnvironmentConfig {
                canvas_width: 800.0,
                canvas_height: 600.0,
                gravity: 0.2,
                jump_velocity: -6.0,
                bird_size: 30.0,
                pipe_width: 60.0,
                pipe_gap: 220.0,
                pipe_spawn_distance: 350.0,
                max_frames_per_game: 1000000,
                grid_size: 20,
                maze_width: 20,
                maze_height: 15,
                wall_thickness: 2,
                movement_speed: 1,
            },
        }
    }
}

impl AIConfig {
    pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| format!("Failed to open config file: {}", e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        serde_yaml::from_str(&contents).map_err(|e| format!("Failed to parse YAML: {}", e))
    }
}
