use ndarray::Array1;
use rand::Rng;
use serde::Serialize;
use std::collections::VecDeque;
use std::path::Path;

use crate::ai::brain::Brain;
use crate::ai::brain::actions::{ActionSelector, ActionStrategy, Action};
use crate::ai::brain::storage::Storage;
use crate::ai::config::AIConfig;

// Cell types in the maze
#[derive(Clone, Copy, PartialEq, Serialize)]
pub enum CellType {
    Empty = 0,
    Wall = 1,
    Player = 2,
    Goal = 3,
}

// Maze representation with walls, player and goal
#[derive(Clone, Serialize)]
pub struct Maze {
    pub cells: Vec<Vec<CellType>>,
    pub width: usize,
    pub height: usize,
}

// Game state for the maze
#[derive(Serialize, Clone)]
pub struct GameState {
    pub maze: Maze,
    pub player_x: usize,
    pub player_y: usize,
    pub goal_x: usize,
    pub goal_y: usize,
    pub score: i32,
    pub game_over: bool,
    pub won: bool,
}

pub struct AI {
    brain: Brain,
    config: AIConfig,
    #[allow(dead_code)]
    storage: Storage,
    action_selector: ActionSelector,
    
    // Game state
    maze: Maze,
    player_x: usize,
    player_y: usize,
    goal_x: usize,
    goal_y: usize,
    score: i32,
    game_over: bool,
    won: bool,
    
    // Environment parameters
    canvas_width: f64,
    canvas_height: f64,
    grid_size: usize,
    maze_width: usize,
    maze_height: usize,
    
    // Training parameters
    frames_since_reset: u32,
    #[allow(dead_code)]
    max_frames_per_game: u32,
    games_played: u32,
    total_reward: f64,
    experience_buffer: VecDeque<(Array1<f64>, usize, f64, Array1<f64>, bool)>, // (state, action_idx, reward, next_state, done)
    previous_distance: f64,
    
    // Control mode
    player_controlled: bool,
    game_speed: f64,
}

impl AI {
    #[allow(dead_code)]
    pub fn new() -> Self {
        // Use default configuration
        let config = AIConfig::default();
        
        Self::new_with_config(config)
    }
    
    pub fn new_from_config<P: AsRef<Path>>(config_path: P) -> Self {
        // Try to load config from file
        let config = match AIConfig::load_from_yaml(config_path) {
            Ok(cfg) => {
                println!("Loaded configuration from file");
                cfg
            },
            Err(e) => {
                eprintln!("Failed to load config: {}", e);
                println!("Using default configuration");
                AIConfig::default()
            }
        };
        
        Self::new_with_config(config)
    }
    
    fn new_with_config(config: AIConfig) -> Self {
        // Setup storage
        let storage = Storage::new(
            "./pkg/brain.nn", 
            "./pkg/config.yaml", 
            "./pkg/training_history.json",
            60 // Autosave every 60 seconds
        );
        
        // Initialize brain - try to load from file first
        let brain = if Path::new("./pkg/brain.nn").exists() {
            match Brain::load("./pkg/brain.nn") {
                Ok(b) => {
                    println!("Loaded neural network from file");
                    b
                },
                Err(e) => {
                    eprintln!("Failed to load brain: {}", e);
                    println!("Creating new neural network");
                    Brain::new(
                        config.network.input_size,
                        config.network.hidden_size1,
                        config.network.hidden_size2,
                        config.network.output_size,
                        config.network.learning_rate
                    )
                }
            }
        } else {
            println!("Creating new neural network");
            Brain::new(
                config.network.input_size,
                config.network.hidden_size1,
                config.network.hidden_size2,
                config.network.output_size,
                config.network.learning_rate
            )
        };
        
        // Create action selector with epsilon-greedy strategy
        let action_selector = ActionSelector::new(ActionStrategy::EpsilonGreedy {
            exploration_rate: config.training.exploration_rate
        });
        
        // Get maze dimensions
        let maze_width = config.environment.maze_width as usize;
        let maze_height = config.environment.maze_height as usize;
        
        // Create an empty maze
        let maze = Maze {
            cells: vec![vec![CellType::Empty; maze_width]; maze_height],
            width: maze_width,
            height: maze_height,
        };
        
        let mut ai = Self {
            brain,
            config: config.clone(),
            storage,
            action_selector,
            
            maze,
            player_x: 1,
            player_y: 1,
            goal_x: maze_width - 2,
            goal_y: maze_height - 2,
            score: 0,
            game_over: false,
            won: false,
            
            canvas_width: config.environment.canvas_width,
            canvas_height: config.environment.canvas_height,
            grid_size: config.environment.grid_size as usize,
            maze_width,
            maze_height,
            
            frames_since_reset: 0,
            max_frames_per_game: config.environment.max_frames_per_game,
            games_played: 0,
            total_reward: 0.0,
            experience_buffer: VecDeque::with_capacity(config.training.buffer_size),
            previous_distance: 0.0,
            
            player_controlled: false,
            game_speed: 1.0,
        };
        
        // Initialize game
        ai.reset_game();
        
        ai
    }
    
    pub fn get_game_state(&self) -> GameState {
        GameState {
            maze: self.maze.clone(),
            player_x: self.player_x,
            player_y: self.player_y,
            goal_x: self.goal_x,
            goal_y: self.goal_y,
            score: self.score,
            game_over: self.game_over,
            won: self.won,
        }
    }
    
    pub fn train_step(&mut self) -> (f64, bool) {
        // Only reset if the game is actually over
        if self.game_over {
            self.reset_game();
            return (0.0, true);
        }
        
        // Increment frame counter
        if self.frames_since_reset < u32::MAX {
            self.frames_since_reset += 1;
        }
        
        // If player-controlled, don't make AI decisions
        if self.player_controlled {
            return (0.0, self.game_over);
        }
        
        // Get current state
        let state = self.get_state_vector();
        
        // Get Q-values from the brain
        let q_values = self.brain.forward(&state);
        
        // Decide action using action selector
        let action = self.action_selector.select_action(&q_values);
        let action_idx = action.to_index();
        
        // Take action
        let reward = self.move_player(&action);
        let done = self.game_over;
        
        // Get new state
        let next_state = self.get_state_vector();
        
        // Store experience
        self.add_experience(state, action_idx, reward, next_state, done);
        
        // Train on a batch of experiences
        self.train_on_batch();
        
        self.total_reward += reward;
        
        // Decay exploration rate
        let current_exploration_rate = self.get_exploration_rate();
        let new_exploration_rate = (current_exploration_rate * self.config.training.exploration_decay)
            .max(self.config.training.min_exploration_rate);
        self.action_selector.update_exploration_rate(new_exploration_rate);
        
        (reward, done)
    }
    
    fn add_experience(&mut self, state: Array1<f64>, action_idx: usize, reward: f64, next_state: Array1<f64>, done: bool) {
        self.experience_buffer.push_back((state, action_idx, reward, next_state, done));
        
        // Limit buffer size
        while self.experience_buffer.len() > self.config.training.buffer_size {
            self.experience_buffer.pop_front();
        }
    }
    
    fn train_on_batch(&mut self) {
        // Skip if buffer is too small
        if self.experience_buffer.len() < self.config.training.batch_size {
            return;
        }
        
        let mut rng = rand::thread_rng();
        
        // Train on a random batch of experiences
        for _ in 0..self.config.training.batch_size.min(64) {
            let index = rng.gen_range(0..self.experience_buffer.len());
            if let Some((state, action_idx, reward, next_state, done)) = self.experience_buffer.get(index) {
                // Prepare target values
                let mut target = Array1::zeros(self.config.network.output_size);
                let current_q_values = self.brain.forward(&state);
                
                // Copy current Q values
                for i in 0..self.config.network.output_size {
                    target[i] = current_q_values[i];
                }
                
                // Q-learning update
                if *done {
                    target[*action_idx] = *reward;
                } else {
                    let next_q_values = self.brain.forward(&next_state);
                    let max_next_q = next_q_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    target[*action_idx] = *reward + self.config.training.discount_factor * max_next_q;
                }
                
                self.brain.train(&state, &target);
            }
        }
    }
    
    fn get_state_vector(&self) -> Array1<f64> {
        let mut state = Array1::zeros(self.config.network.input_size);
        
        // Normalized player position
        state[0] = self.player_x as f64 / self.maze_width as f64;
        state[1] = self.player_y as f64 / self.maze_height as f64;
        
        // Normalized goal position
        state[2] = self.goal_x as f64 / self.maze_width as f64;
        state[3] = self.goal_y as f64 / self.maze_height as f64;
        
        // Direction to goal (normalized)
        let dx = self.goal_x as f64 - self.player_x as f64;
        let dy = self.goal_y as f64 - self.player_y as f64;
        let _distance = (dx * dx + dy * dy).sqrt(); // Prefixed with underscore to avoid warning
        
        state[4] = dx / (self.maze_width as f64);
        state[5] = dy / (self.maze_height as f64);
        
        // Walls in four directions (up, down, left, right)
        state[6] = if self.is_wall(self.player_x, self.player_y - 1) { 1.0 } else { 0.0 };
        state[7] = if self.is_wall(self.player_x, self.player_y + 1) { 1.0 } else { 0.0 };
        state[8] = if self.is_wall(self.player_x - 1, self.player_y) { 1.0 } else { 0.0 };
        state[9] = if self.is_wall(self.player_x + 1, self.player_y) { 1.0 } else { 0.0 };
        
        state
    }
    
    fn is_wall(&self, x: usize, y: usize) -> bool {
        if x >= self.maze_width || y >= self.maze_height {
            return true; // Out of bounds is a wall
        }
        self.maze.cells[y][x] == CellType::Wall
    }
    
    fn move_player(&mut self, action: &Action) -> f64 {
        let mut reward = -0.01; // Small penalty for each move to encourage efficiency
        
        // Get the movement direction
        let (dx, dy) = action.to_direction();
        
        // Calculate new position
        let new_x = self.player_x as i32 + dx;
        let new_y = self.player_y as i32 + dy;
        
        // Check if the new position is valid
        if new_x >= 0 && new_x < self.maze_width as i32 && 
           new_y >= 0 && new_y < self.maze_height as i32 {
            
            let new_x = new_x as usize;
            let new_y = new_y as usize;
            
            match self.maze.cells[new_y][new_x] {
                CellType::Wall => {
                    // Hit a wall
                    reward = -0.5;
                },
                CellType::Goal => {
                    // Reached the goal
                    reward = 10.0;
                    self.score += 100;
                    self.game_over = true;
                    self.won = true;
                },
                _ => {
                    // Valid move
                    self.maze.cells[self.player_y][self.player_x] = CellType::Empty;
                    self.player_x = new_x;
                    self.player_y = new_y;
                    self.maze.cells[new_y][new_x] = CellType::Player;
                    
                    // Calculate distance to goal
                    let dx = self.goal_x as f64 - new_x as f64;
                    let dy = self.goal_y as f64 - new_y as f64;
                    let new_distance = (dx * dx + dy * dy).sqrt();
                    
                    // Reward for getting closer to the goal
                    if new_distance < self.previous_distance {
                        reward += 0.1;
                    }
                    
                    self.previous_distance = new_distance;
                }
            }
        } else {
            // Out of bounds
            reward = -0.5;
        }
        
        // Check for timeout
        if self.frames_since_reset > 500 {
            self.game_over = true;
            reward = -1.0;
        }
        
        reward
    }
    
    pub fn manual_move(&mut self, action: &Action) -> f64 {
        if self.game_over {
            return 0.0;
        }
        
        // Record state before action
        let state = self.get_state_vector();
        
        // Execute move
        let action_idx = action.to_index();
        let reward = self.move_player(action);
        let done = self.game_over;
        
        // Get new state
        let next_state = self.get_state_vector();
        
        // Add this experience for learning
        self.add_experience(state, action_idx, reward, next_state, done);
        
        // Train on this experience
        self.train_on_batch();
        
        if self.frames_since_reset < u32::MAX {
            self.frames_since_reset += 1;
        }
        self.total_reward += reward;
        
        reward
    }
    
    fn reset_game(&mut self) {
        if self.game_over {
            self.games_played += 1;
            println!("Game {}: Score = {}, Won = {}, Total Reward = {:.2}", 
                    self.games_played, self.score, self.won, self.total_reward);
            self.total_reward = 0.0;
        }
        
        // Generate a new maze
        self.generate_maze();
        
        // Set start and goal positions
        self.player_x = 1;
        self.player_y = 1;
        self.goal_x = self.maze_width - 2;
        self.goal_y = self.maze_height - 2;
        
        // Clear player cell and set player position
        self.maze.cells[self.player_y][self.player_x] = CellType::Player;
        
        // Place goal
        self.maze.cells[self.goal_y][self.goal_x] = CellType::Goal;
        
        // Reset game state
        self.score = 0;
        self.game_over = false;
        self.won = false;
        self.frames_since_reset = 0;
        
        // Calculate initial distance to goal
        let dx = self.goal_x as f64 - self.player_x as f64;
        let dy = self.goal_y as f64 - self.player_y as f64;
        self.previous_distance = (dx * dx + dy * dy).sqrt();
    }
    
    // Simple maze generation using recursive division
    fn generate_maze(&mut self) {
        // Start with an empty grid
        self.maze.cells = vec![vec![CellType::Empty; self.maze_width]; self.maze_height];
        
        // Create border walls
        for x in 0..self.maze_width {
            self.maze.cells[0][x] = CellType::Wall;
            self.maze.cells[self.maze_height - 1][x] = CellType::Wall;
        }
        
        for y in 0..self.maze_height {
            self.maze.cells[y][0] = CellType::Wall;
            self.maze.cells[y][self.maze_width - 1] = CellType::Wall;
        }
        
        // Add some random interior walls
        let mut rng = rand::thread_rng();
        let wall_count = (self.maze_width * self.maze_height) / 8;
        
        for _ in 0..wall_count {
            let x = rng.gen_range(1..self.maze_width - 1);
            let y = rng.gen_range(1..self.maze_height - 1);
            
            // Don't block start or end
            if (x == 1 && y == 1) || (x == self.maze_width - 2 && y == self.maze_height - 2) {
                continue;
            }
            
            self.maze.cells[y][x] = CellType::Wall;
        }
        
        // Make sure there is a path from start to goal
        self.ensure_path_exists();
    }
    
    // Simple implementation to ensure a path exists
    fn ensure_path_exists(&mut self) {
        // For simplicity, just clear a direct path with some randomness
        let mut x = 1;
        let mut y = 1;
        
        let mut rng = rand::thread_rng();
        
        while x < self.goal_x || y < self.goal_y {
            // Clear current cell
            self.maze.cells[y][x] = CellType::Empty;
            
            // Move toward goal with some randomness
            if x < self.goal_x && (y == self.goal_y || rng.gen_bool(0.7)) {
                x += 1;
            } else if y < self.goal_y {
                y += 1;
            }
            
            // Clear the cell
            self.maze.cells[y][x] = CellType::Empty;
        }
    }
    
    // Save the neural network to a file
    pub fn save_brain<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.brain.save(path)
    }
    
    // Get current exploration rate
    pub fn get_exploration_rate(&self) -> f64 {
        self.action_selector.get_exploration_rate()
    }
    
    // Set whether the game is currently player-controlled
    pub fn set_player_controlled(&mut self, is_player_controlled: bool) {
        self.player_controlled = is_player_controlled;
        println!("Control mode changed to: {}", 
                 if is_player_controlled { "Player" } else { "AI" });
    }
    
    // Set game speed
    pub fn set_game_speed(&mut self, speed: f64) {
        // Clamp speed to reasonable values
        self.game_speed = speed.max(0.5).min(2.0);
        println!("Game speed set to: {}", self.game_speed);
    }
    
    // Get current game speed
    pub fn get_game_speed(&self) -> f64 {
        self.game_speed
    }
}
