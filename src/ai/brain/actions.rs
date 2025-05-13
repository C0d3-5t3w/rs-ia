use ndarray::Array1;
use rand::Rng;

pub enum Action {
    Up,
    Down,
    Left,
    Right,
}

pub enum ActionStrategy {
    EpsilonGreedy {
        exploration_rate: f64,
    },
    #[allow(dead_code)]
    Softmax {
        temperature: f64,
    },
    #[allow(dead_code)]
    Greedy,
}

pub struct ActionSelector {
    strategy: ActionStrategy,
}

impl ActionSelector {
    pub fn new(strategy: ActionStrategy) -> Self {
        Self { strategy }
    }

    pub fn select_action(&self, q_values: &Array1<f64>) -> Action {
        match &self.strategy {
            ActionStrategy::EpsilonGreedy { exploration_rate } => {
                let mut rng = rand::thread_rng();

                if rng.gen_range(0.0..1.0) < *exploration_rate {
                    match rng.gen_range(0..4) {
                        0 => Action::Up,
                        1 => Action::Down,
                        2 => Action::Left,
                        _ => Action::Right,
                    }
                } else {
                    let mut best_action = 0;
                    let mut best_value = q_values[0];

                    for i in 1..q_values.len() {
                        if q_values[i] > best_value {
                            best_value = q_values[i];
                            best_action = i;
                        }
                    }

                    match best_action {
                        0 => Action::Up,
                        1 => Action::Down,
                        2 => Action::Left,
                        _ => Action::Right,
                    }
                }
            }

            ActionStrategy::Softmax { temperature } => {
                let mut rng = rand::thread_rng();

                let mut exp_values = vec![0.0; q_values.len()];
                let mut sum_exp = 0.0;

                for i in 0..q_values.len() {
                    exp_values[i] = (q_values[i] / temperature).exp();
                    sum_exp += exp_values[i];
                }

                let mut probs = vec![0.0; q_values.len()];
                for i in 0..q_values.len() {
                    probs[i] = exp_values[i] / sum_exp;
                }

                let r = rng.gen_range(0.0..1.0);
                let mut cumulative_prob = 0.0;

                for i in 0..probs.len() {
                    cumulative_prob += probs[i];
                    if r <= cumulative_prob {
                        match i {
                            0 => return Action::Up,
                            1 => return Action::Down,
                            2 => return Action::Left,
                            _ => return Action::Right,
                        }
                    }
                }

                Action::Right
            }

            ActionStrategy::Greedy => {
                let mut best_action = 0;
                let mut best_value = q_values[0];

                for i in 1..q_values.len() {
                    if q_values[i] > best_value {
                        best_value = q_values[i];
                        best_action = i;
                    }
                }

                match best_action {
                    0 => Action::Up,
                    1 => Action::Down,
                    2 => Action::Left,
                    _ => Action::Right,
                }
            }
        }
    }

    pub fn update_exploration_rate(&mut self, new_rate: f64) {
        if let ActionStrategy::EpsilonGreedy {
            ref mut exploration_rate,
        } = self.strategy
        {
            *exploration_rate = new_rate;
        }
    }

    pub fn get_exploration_rate(&self) -> f64 {
        match &self.strategy {
            ActionStrategy::EpsilonGreedy { exploration_rate } => *exploration_rate,
            _ => 0.0,
        }
    }
}

impl Action {
    pub fn to_direction(&self) -> (i32, i32) {
        match self {
            Action::Up => (0, -1),
            Action::Down => (0, 1),
            Action::Left => (-1, 0),
            Action::Right => (1, 0),
        }
    }

    pub fn to_index(&self) -> usize {
        match self {
            Action::Up => 0,
            Action::Down => 1,
            Action::Left => 2,
            Action::Right => 3,
        }
    }
}
