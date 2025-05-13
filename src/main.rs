mod ai;
mod ws;

use std::sync::{Arc, Mutex};
use ai::AI;
use ai::brain::orchestrator::Orchestrator;

#[tokio::main]
async fn main() {
    println!("Starting Flappy Bird AI Training Server");
    
    // Create AI instance with configuration from YAML
    let ai = Arc::new(Mutex::new(AI::new_from_config("./pkg/config.yaml")));
    
    // Create orchestrator for training management
    let orchestrator = Orchestrator::new(ai.clone());
    
    // Start training in a background task
    let _training_ai = ai.clone();
    tokio::spawn(async move {
        orchestrator.start_training_loop().await;
    });
    
    // Auto-save neural network periodically
    let save_ai = ai.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            {
                let ai_guard = save_ai.lock().unwrap();
                ai_guard.save_brain("./pkg/brain.nn").unwrap_or_else(|e| {
                    eprintln!("Error saving brain: {}", e);
                });
            }
            println!("Neural network saved to ./pkg/brain.nn");
        }
    });
    
    // Start WebSocket server
    ws::start_server(ai).await;
}
