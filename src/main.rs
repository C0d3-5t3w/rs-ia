mod ai;
mod ws;

use ai::AI;
use ai::brain::orchestrator::Orchestrator;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    println!("Starting Maze Navigator AI Training Server");

    let ai = Arc::new(Mutex::new(AI::new_from_config("./pkg/config.yaml")));

    let orchestrator = Orchestrator::new(ai.clone());

    let _training_ai = ai.clone();
    tokio::spawn(async move {
        orchestrator.start_training_loop().await;
    });

    let save_ai = ai.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            {
                match save_ai.lock() {
                    Ok(ai_guard) => {
                        ai_guard.save_brain("./pkg/brain.nn").unwrap_or_else(|e| {
                            eprintln!("Error saving brain: {}", e);
                        });
                    }
                    Err(poisoned) => {
                        poisoned
                            .into_inner()
                            .save_brain("./pkg/brain.nn")
                            .unwrap_or_else(|e| {
                                eprintln!("Error saving brain: {}", e);
                            });
                    }
                }
                println!("Neural network saved to ./pkg/brain.nn");
            }
        }
    });

    ws::start_server(ai).await;
}
