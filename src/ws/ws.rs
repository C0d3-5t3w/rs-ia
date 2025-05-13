use futures::{FutureExt, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{self, Filter};

use crate::ai::AI;

type Clients = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<warp::ws::Message>>>>;

pub async fn start_server(ai: Arc<Mutex<AI>>) {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    let clients_filter = warp::any().map(move || clients.clone());
    let ai_filter = warp::any().map(move || ai.clone());

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(clients_filter)
        .and(ai_filter)
        .map(|ws: warp::ws::Ws, clients, ai| {
            ws.on_upgrade(move |socket| handle_connection(socket, clients, ai))
        });

    let static_route = warp::path("static").and(warp::fs::dir("./src/ws/web/assets"));

    let index_route = warp::path::end().map(|| {
        let html = include_str!("web/pages/index.html");
        warp::reply::html(html)
    });

    let routes = ws_route.or(static_route).or(index_route);

    println!("Server started at http://localhost:8080");
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

async fn handle_connection(ws: warp::ws::WebSocket, clients: Clients, ai: Arc<Mutex<AI>>) {
    println!("New WebSocket connection");

    let (ws_tx, mut ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();

    let rx_stream = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(rx_stream.map(Ok).forward(ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("WebSocket send error: {}", e);
        }
    }));

    let client_id = rand::random::<u64>().to_string();

    clients.lock().unwrap().insert(client_id.clone(), tx);

    let clients_clone = clients.clone();
    let ai_clone = ai.clone();
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
        loop {
            interval.tick().await;

            let game_state = {
                match ai_clone.lock() {
                    Ok(ai_locked) => ai_locked.get_game_state(),
                    Err(poisoned) => {
                        eprintln!("Warning: AI mutex was poisoned. Recovering...");
                        poisoned.into_inner().get_game_state()
                    }
                }
            };

            let json = match serde_json::to_string(&game_state) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error serializing game state: {}", e);
                    continue;
                }
            };

            let message = warp::ws::Message::text(json);

            let clients_locked = match clients_clone.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    eprintln!("Warning: Clients mutex was poisoned. Recovering...");
                    poisoned.into_inner()
                }
            };

            for (_id, client_tx) in clients_locked.iter() {
                if let Err(_) = client_tx.send(message.clone()) {}
            }
        }
    });

    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_close() {
                    break;
                }

                if let Ok(text) = msg.to_str() {
                    if let Ok(command) = serde_json::from_str::<serde_json::Value>(text) {
                        if let Some(action) = command.get("action") {
                            if let Some(action_str) = action.as_str() {
                                match action_str {
                                    "up" => match ai.lock() {
                                        Ok(mut ai_guard) => {
                                            ai_guard.manual_move(
                                                &crate::ai::brain::actions::Action::Up,
                                            );
                                        }
                                        Err(poisoned) => {
                                            poisoned.into_inner().manual_move(
                                                &crate::ai::brain::actions::Action::Up,
                                            );
                                        }
                                    },
                                    "down" => match ai.lock() {
                                        Ok(mut ai_guard) => {
                                            ai_guard.manual_move(
                                                &crate::ai::brain::actions::Action::Down,
                                            );
                                        }
                                        Err(poisoned) => {
                                            poisoned.into_inner().manual_move(
                                                &crate::ai::brain::actions::Action::Down,
                                            );
                                        }
                                    },
                                    "left" => match ai.lock() {
                                        Ok(mut ai_guard) => {
                                            ai_guard.manual_move(
                                                &crate::ai::brain::actions::Action::Left,
                                            );
                                        }
                                        Err(poisoned) => {
                                            poisoned.into_inner().manual_move(
                                                &crate::ai::brain::actions::Action::Left,
                                            );
                                        }
                                    },
                                    "right" => match ai.lock() {
                                        Ok(mut ai_guard) => {
                                            ai_guard.manual_move(
                                                &crate::ai::brain::actions::Action::Right,
                                            );
                                        }
                                        Err(poisoned) => {
                                            poisoned.into_inner().manual_move(
                                                &crate::ai::brain::actions::Action::Right,
                                            );
                                        }
                                    },
                                    "getSettings" => {
                                        let settings = {
                                            match ai.lock() {
                                                Ok(ai_guard) => serde_json::json!({
                                                    "settings": {
                                                        "gameSpeed": ai_guard.get_game_speed()
                                                    }
                                                }),
                                                Err(poisoned) => {
                                                    let ai_guard = poisoned.into_inner();
                                                    serde_json::json!({
                                                        "settings": {
                                                            "gameSpeed": ai_guard.get_game_speed()
                                                        }
                                                    })
                                                }
                                            }
                                        };

                                        match clients.lock() {
                                            Ok(client) => {
                                                if let Some(client_tx) = client.get(&client_id) {
                                                    let _ =
                                                        client_tx.send(warp::ws::Message::text(
                                                            serde_json::to_string(&settings)
                                                                .unwrap(),
                                                        ));
                                                }
                                            }
                                            Err(poisoned) => {
                                                let client = poisoned.into_inner();
                                                if let Some(client_tx) = client.get(&client_id) {
                                                    let _ =
                                                        client_tx.send(warp::ws::Message::text(
                                                            serde_json::to_string(&settings)
                                                                .unwrap(),
                                                        ));
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        } else if let Some(control_mode) = command.get("controlMode") {
                            if let Some(is_player) = control_mode.as_bool() {
                                match ai.lock() {
                                    Ok(mut ai_guard) => {
                                        ai_guard.set_player_controlled(is_player);
                                    }
                                    Err(poisoned) => {
                                        poisoned.into_inner().set_player_controlled(is_player);
                                    }
                                }
                            }
                        } else if let Some(game_speed) = command.get("gameSpeed") {
                            if let Some(speed_value) = game_speed.as_f64() {
                                match ai.lock() {
                                    Ok(mut ai_guard) => {
                                        ai_guard.set_game_speed(speed_value);
                                    }
                                    Err(poisoned) => {
                                        poisoned.into_inner().set_game_speed(speed_value);
                                    }
                                }
                            }
                        } else if let Some(difficulty) = command.get("difficulty") {
                            if let Some(difficulty_value) = difficulty.as_str() {
                                match ai.lock() {
                                    Ok(mut ai_guard) => {
                                        println!(
                                            "Note: Difficulty setting '{}' received but not implemented",
                                            difficulty_value
                                        );
                                    }
                                    Err(poisoned) => {
                                        println!(
                                            "Note: Difficulty setting '{}' received but not implemented",
                                            difficulty_value
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    match clients.lock() {
        Ok(mut clients_guard) => {
            clients_guard.remove(&client_id);
        }
        Err(poisoned) => {
            let mut clients_guard = poisoned.into_inner();
            clients_guard.remove(&client_id);
        }
    }
    println!("WebSocket client disconnected: {}", client_id);
}
