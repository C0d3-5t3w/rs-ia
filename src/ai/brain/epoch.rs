use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

// Individual training epoch stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochStats {
    pub epoch_id: u64,
    pub timestamp: DateTime<Local>,
    pub games_played: u32,
    pub max_score: u32,
    pub avg_score: f64,
    pub total_reward: f64,
    pub avg_game_length: f64,
    pub exploration_rate: f64, 
}

// Track training progress over multiple epochs
#[derive(Debug, Serialize, Deserialize)]
pub struct EpochTracker {
    pub epochs: VecDeque<EpochStats>,
    pub current_epoch: u64,
    pub best_score: u32,
    pub max_history_size: usize,
}

impl EpochTracker {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            epochs: VecDeque::with_capacity(max_history_size),
            current_epoch: 0,
            best_score: 0,
            max_history_size: max_history_size,
        }
    }
    
    #[allow(dead_code)]
    pub fn record_epoch(&mut self, 
                        games_played: u32, 
                        max_score: u32, 
                        avg_score: f64,
                        total_reward: f64, 
                        avg_game_length: f64, 
                        exploration_rate: f64) {
        
        // Update best score if needed
        if max_score > self.best_score {
            self.best_score = max_score;
        }
        
        // Create epoch stats
        let stats = EpochStats {
            epoch_id: self.current_epoch,
            timestamp: Local::now(),
            games_played,
            max_score,
            avg_score,
            total_reward,
            avg_game_length,
            exploration_rate,
        };
        
        // Add to history
        self.epochs.push_back(stats);
        
        // Limit history size
        if self.epochs.len() > self.max_history_size {
            self.epochs.pop_front();
        }
        
        // Log info
        println!("Epoch {}: Avg Score: {:.2}, Max Score: {}, Games: {}, Exploration: {:.3}", 
                 self.current_epoch, avg_score, max_score, games_played, exploration_rate);
        
        // Increment epoch counter
        self.current_epoch += 1;
    }
    
    // Check if performance is improving
    #[allow(dead_code)]
    pub fn is_improving(&self) -> bool {
        if self.epochs.len() < 5 {
            return true; // Not enough data
        }
        
        // Get average score of last 5 epochs
        let recent_epochs: Vec<&EpochStats> = self.epochs.iter().rev().take(5).collect();
        let recent_avg: f64 = recent_epochs.iter().map(|e| e.avg_score).sum::<f64>() / 5.0;
        
        // Get average score of 5 epochs before that
        if self.epochs.len() < 10 {
            return true; // Not enough historical data
        }
        
        let previous_epochs: Vec<&EpochStats> = self.epochs.iter().rev().skip(5).take(5).collect();
        let previous_avg: f64 = previous_epochs.iter().map(|e| e.avg_score).sum::<f64>() / 5.0;
        
        recent_avg > previous_avg
    }
    
    // Save training history to a file
    #[allow(dead_code)]
    pub fn save_history<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let serialized = serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize epoch history: {}", e))?;
        
        let mut file = File::create(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        
        file.write_all(serialized.as_bytes())
            .map_err(|e| format!("Failed to write to file: {}", e))?;
        
        Ok(())
    }
    
    // Load training history from a file
    #[allow(dead_code)]
    pub fn load_history<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to deserialize epoch history: {}", e))
    }
}
