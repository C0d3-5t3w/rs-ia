use ndarray::{Array, Array1, Array2};
use rand::thread_rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Brain {
    weights1: Array2<f64>,
    biases1: Array1<f64>,
    weights2: Array2<f64>,
    biases2: Array1<f64>,
    weights3: Array2<f64>,
    biases3: Array1<f64>,
    input_size: usize,
    hidden_size1: usize,
    hidden_size2: usize,
    output_size: usize,
    learning_rate: f64,
}

impl Brain {
    pub fn new(input_size: usize, hidden_size1: usize, hidden_size2: usize, output_size: usize, learning_rate: f64) -> Self {
        let mut rng = thread_rng();
        
        // Initialize with small random values
        let normal = Normal::new(0.0, 0.1).unwrap();
        
        let weights1 = Array::from_shape_fn((hidden_size1, input_size), |_| normal.sample(&mut rng));
        let biases1 = Array::from_shape_fn(hidden_size1, |_| normal.sample(&mut rng));
        let weights2 = Array::from_shape_fn((hidden_size2, hidden_size1), |_| normal.sample(&mut rng));
        let biases2 = Array::from_shape_fn(hidden_size2, |_| normal.sample(&mut rng));
        let weights3 = Array::from_shape_fn((output_size, hidden_size2), |_| normal.sample(&mut rng));
        let biases3 = Array::from_shape_fn(output_size, |_| normal.sample(&mut rng));
        
        Self {
            weights1,
            biases1,
            weights2,
            biases2,
            weights3,
            biases3,
            input_size,
            hidden_size1,
            hidden_size2,
            output_size,
            learning_rate,
        }
    }
    
    // Forward pass through the network
    pub fn forward(&self, input: &Array1<f64>) -> Array1<f64> {
        let hidden1 = self.weights1.dot(input) + &self.biases1;
        let hidden1_activated = self.leaky_relu(&hidden1);
        let hidden2 = self.weights2.dot(&hidden1_activated) + &self.biases2;
        let hidden2_activated = self.leaky_relu(&hidden2);
        let output = self.weights3.dot(&hidden2_activated) + &self.biases3;
        self.sigmoid(&output)
    }
    
    // ReLU activation function
    fn relu(&self, x: &Array1<f64>) -> Array1<f64> {
        x.mapv(|val| if val > 0.0 { val } else { 0.0 })
    }
    
    // Leaky ReLU activation function for better gradient flow
    fn leaky_relu(&self, x: &Array1<f64>) -> Array1<f64> {
        x.mapv(|val| if val > 0.0 { val } else { 0.01 * val })
    }
    
    // Sigmoid activation function
    fn sigmoid(&self, x: &Array1<f64>) -> Array1<f64> {
        x.mapv(|val| 1.0 / (1.0 + (-val).exp()))
    }
    
    // Enhanced training function with improved backpropagation
    pub fn train(&mut self, input: &Array1<f64>, target: &Array1<f64>) {
        // Forward pass
        let hidden1 = self.weights1.dot(input) + &self.biases1;
        let hidden1_activated = self.leaky_relu(&hidden1);
        let hidden2 = self.weights2.dot(&hidden1_activated) + &self.biases2;
        let hidden2_activated = self.leaky_relu(&hidden2);
        let output = self.weights3.dot(&hidden2_activated) + &self.biases3;
        let output_activated = self.sigmoid(&output);
        
        // Backpropagation - output layer
        let output_error = target - &output_activated;
        let output_delta = &output_error * &output_activated * &(1.0 - &output_activated);
        
        // Backpropagation - hidden layer 2
        let hidden2_error = self.weights3.t().dot(&output_delta);
        let hidden2_delta = &hidden2_error * &hidden2_activated.mapv(|h| if h > 0.0 { 1.0 } else { 0.01 });
        
        // Backpropagation - hidden layer 1
        let hidden1_error = self.weights2.t().dot(&hidden2_delta);
        let hidden1_delta = &hidden1_error * &hidden1_activated.mapv(|h| if h > 0.0 { 1.0 } else { 0.01 });
        
        // Update weights and biases
        // Output layer updates
        let output_delta_reshaped = output_delta.clone().into_shape((output_delta.len(), 1)).unwrap();
        let hidden2_activated_reshaped = hidden2_activated.clone().into_shape((1, hidden2_activated.len())).unwrap();
        self.weights3 = &self.weights3 + &(&output_delta_reshaped.dot(&hidden2_activated_reshaped) * self.learning_rate);
        self.biases3 = &self.biases3 + &(&output_delta * self.learning_rate);
        
        // Hidden layer 2 updates
        let hidden2_delta_reshaped = hidden2_delta.clone().into_shape((hidden2_delta.len(), 1)).unwrap();
        let hidden1_activated_reshaped = hidden1_activated.clone().into_shape((1, hidden1_activated.len())).unwrap();
        self.weights2 = &self.weights2 + &(&hidden2_delta_reshaped.dot(&hidden1_activated_reshaped) * self.learning_rate);
        self.biases2 = &self.biases2 + &(&hidden2_delta * self.learning_rate);
        
        // Hidden layer 1 updates
        let hidden1_delta_reshaped = hidden1_delta.clone().into_shape((hidden1_delta.len(), 1)).unwrap();
        let input_reshaped = input.clone().into_shape((1, input.len())).unwrap();
        self.weights1 = &self.weights1 + &(&hidden1_delta_reshaped.dot(&input_reshaped) * self.learning_rate);
        self.biases1 = &self.biases1 + &(&hidden1_delta * self.learning_rate);
    }

    // Save the neural network to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let serialized = serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize brain: {}", e))?;
        
        let mut file = File::create(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        
        file.write_all(serialized.as_bytes())
            .map_err(|e| format!("Failed to write to file: {}", e))?;
        
        Ok(())
    }

    // Load a neural network from a file
    #[allow(dead_code)]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to deserialize brain: {}", e))
    }
    
    // Get network architecture info
    #[allow(dead_code)]
    pub fn get_architecture(&self) -> (usize, usize, usize, usize) {
        (self.input_size, self.hidden_size1, self.hidden_size2, self.output_size)
    }
    
    // Set learning rate (useful for adjusting during training)
    #[allow(dead_code)]
    pub fn set_learning_rate(&mut self, learning_rate: f64) {
        self.learning_rate = learning_rate;
    }
}
