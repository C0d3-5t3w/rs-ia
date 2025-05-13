use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::ai::AI;
use crate::ai::brain::epoch::EpochTracker;

pub struct Orchestrator {
    ai: Arc<Mutex<AI>>,
    #[allow(dead_code)]
    epoch_tracker: EpochTracker,
    games_per_epoch: u32,
    training_speed: TrainingSpeed,
}

pub enum TrainingSpeed {
    #[allow(dead_code)]
    Slow,    // Train with delays to make visualization easier
    #[allow(dead_code)]
    Fast,    // Train as fast as possible
    Adaptive // Adjust speed based on whether someone is viewing
}

impl Orchestrator {
    pub fn new(ai: Arc<Mutex<AI>>) -> Self {
        Self {
            ai,
            epoch_tracker: EpochTracker::new(100),
            games_per_epoch: 10,
            training_speed: TrainingSpeed::Adaptive,
        }
    }
    
    pub async fn start_training_loop(&self) {
        let mut epoch_stats = Vec::new();
        let mut epoch_game_count = 0;
        let mut total_rewards = 0.0;
        let mut max_score = 0;
        let mut total_score = 0;
        let mut total_frames = 0;
        
        println!("Starting AI training loop");
        
        loop {
            let _start_time = Instant::now();
            let mut game_done = false;
            let mut game_reward = 0.0;
            let mut frame_count = 0;
            
            // Run a complete game
            while !game_done {
                // Lock AI and perform a training step
                {
                    let mut ai_guard = self.ai.lock().unwrap();
                    let (reward, done) = ai_guard.train_step();
                    
                    game_reward += reward;
                    game_done = done;
                    frame_count += 1;
                    
                    // Get current score for tracking
                    if done {
                        let score = ai_guard.get_game_state().score;
                        max_score = max_score.max(score);
                        total_score += score;
                    }
                }
                
                // Apply training speed control
                match self.training_speed {
                    TrainingSpeed::Slow => {
                        tokio::time::sleep(Duration::from_millis(20)).await;
                    },
                    TrainingSpeed::Fast => {
                        // No delay, train as fast as possible
                    },
                    TrainingSpeed::Adaptive => {
                        // Check if there are any viewers connected
                        // For now, just use a small delay
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                }
            }
            
            // Record game stats
            epoch_stats.push((game_reward, frame_count));
            epoch_game_count += 1;
            total_rewards += game_reward;
            total_frames += frame_count;
            
            // Check if epoch is complete
            if epoch_game_count >= self.games_per_epoch {
                // Calculate epoch statistics
                let avg_score = total_score as f64 / epoch_game_count as f64;
                let avg_game_length = total_frames as f64 / epoch_game_count as f64;
                
                // Get current exploration rate
                let exploration_rate = {
                    let ai_guard = self.ai.lock().unwrap();
                    ai_guard.get_exploration_rate()
                };
                
                // Record epoch
                // Note: In a real implementation, we'd create a mutable epoch_tracker
                // For this example, just print the stats
                println!(
                    "Epoch complete: Games={}, Max Score={}, Avg Score={:.2}, Total Reward={:.2}, Avg Length={:.2}, Exploration={:.3}",
                    epoch_game_count, max_score, avg_score, total_rewards, avg_game_length, exploration_rate
                );
                
                // Reset for next epoch
                epoch_stats.clear();
                epoch_game_count = 0;
                total_rewards = 0.0;
                max_score = 0;
                total_score = 0;
                total_frames = 0;
                
                // Save neural network periodically
                {
                    let ai_guard = self.ai.lock().unwrap();
                    if let Err(e) = ai_guard.save_brain("./pkg/brain.nn") {
                        eprintln!("Failed to save brain: {}", e);
                    } else {
                        println!("Saved neural network to ./pkg/brain.nn");
                    }
                }
            }
        }
    }
    
    #[allow(dead_code)]
    pub fn set_training_speed(&mut self, speed: TrainingSpeed) {
        self.training_speed = speed;
    }
    
    #[allow(dead_code)]
    pub fn set_games_per_epoch(&mut self, games: u32) {
        self.games_per_epoch = games;
    }
}
