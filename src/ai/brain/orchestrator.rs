use crate::ai::AI;
use crate::ai::brain::epoch::EpochTracker;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct Orchestrator {
    ai: Arc<Mutex<AI>>,
    #[allow(dead_code)]
    epoch_tracker: EpochTracker,
    games_per_epoch: u32,
    training_speed: TrainingSpeed,
}

pub enum TrainingSpeed {
    #[allow(dead_code)]
    Slow,
    #[allow(dead_code)]
    Fast,
    Adaptive,
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

            while !game_done {
                {
                    let mut ai_guard = self.ai.lock().unwrap();
                    let (reward, done) = ai_guard.train_step();

                    game_reward += reward;
                    game_done = done;
                    frame_count += 1;

                    if done {
                        let score = ai_guard.get_game_state().score;
                        max_score = max_score.max(score);
                        total_score += score;
                    }
                }

                match self.training_speed {
                    TrainingSpeed::Slow => {
                        tokio::time::sleep(Duration::from_millis(20)).await;
                    }
                    TrainingSpeed::Fast => {}
                    TrainingSpeed::Adaptive => {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                }
            }

            epoch_stats.push((game_reward, frame_count));
            epoch_game_count += 1;
            total_rewards += game_reward;
            total_frames += frame_count;

            if epoch_game_count >= self.games_per_epoch {
                let avg_score = total_score as f64 / epoch_game_count as f64;
                let avg_game_length = total_frames as f64 / epoch_game_count as f64;

                let exploration_rate = {
                    let ai_guard = self.ai.lock().unwrap();
                    ai_guard.get_exploration_rate()
                };

                println!(
                    "Epoch complete: Games={}, Max Score={}, Avg Score={:.2}, Total Reward={:.2}, Avg Length={:.2}, Exploration={:.3}",
                    epoch_game_count,
                    max_score,
                    avg_score,
                    total_rewards,
                    avg_game_length,
                    exploration_rate
                );

                epoch_stats.clear();
                epoch_game_count = 0;
                total_rewards = 0.0;
                max_score = 0;
                total_score = 0;
                total_frames = 0;

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
