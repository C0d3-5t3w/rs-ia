// Maze Game Renderer

interface MazeCell {
    cells: number[][];
    width: number;
    height: number;
}

interface GameStateData {
    maze: MazeCell;
    player_x: number;
    player_y: number;
    goal_x: number;
    goal_y: number;
    score: number;
    game_over: boolean;
    won: boolean;
}

interface CellTypes {
    EMPTY: number;
    WALL: number;
    PLAYER: number;
    GOAL: number;
}

interface ColorScheme {
    background: string;
    wall: string;
    player: string;
    goal: string;
    empty: string;
    text: string;
}

class MazeGameRenderer {
    private canvas: HTMLCanvasElement;
    private ctx: CanvasRenderingContext2D;
    private width: number;
    private height: number;
    
    // Cell types
    private CELL_TYPES: CellTypes;
    
    // Game state
    private gameState: GameStateData;
    
    // Colors
    private colors: ColorScheme;
    
    // Game control options
    private playerControlled: boolean;
    private showGrid: boolean;
    private gameSpeed: number;
    
    // WebSocket connection
    private socket: WebSocket | null;
    
    constructor() {
        const canvasElement = document.getElementById('game');
        if (!canvasElement || !(canvasElement instanceof HTMLCanvasElement)) {
            throw new Error("Canvas element not found");
        }
        
        this.canvas = canvasElement;
        const context = this.canvas.getContext('2d');
        if (!context) {
            throw new Error("Could not get 2D context");
        }
        
        this.ctx = context;
        this.width = this.canvas.width;
        this.height = this.canvas.height;
        
        // Cell types
        this.CELL_TYPES = {
            EMPTY: 0,
            WALL: 1,
            PLAYER: 2,
            GOAL: 3
        };
        
        // Game state
        this.gameState = {
            maze: {
                cells: [],
                width: 20,
                height: 15
            },
            player_x: 1,
            player_y: 1,
            goal_x: 18,
            goal_y: 13,
            score: 0,
            game_over: false,
            won: false
        };
        
        // Colors
        this.colors = {
            background: '#f0f0f0',
            wall: '#333333',
            player: '#4285F4',
            goal: '#EA4335',
            empty: '#FFFFFF',
            text: '#000000'
        };
        
        // Game control options
        this.playerControlled = false;
        this.showGrid = true;
        this.gameSpeed = 0.7;
        
        // Initialize socket as null
        this.socket = null;
        
        // Connect to WebSocket
        this.connectWebSocket();
        
        // Setup keyboard controls
        this.setupControls();
        
        // Start rendering
        this.render();
        
        // Create UI controls
        this.createUI();
    }
    
    createUI(): void {
        // Create control panel
        const controlPanel = document.createElement('div');
        controlPanel.className = 'control-panel';
        document.body.appendChild(controlPanel);
        
        // Toggle player control
        const controlToggle = document.createElement('button');
        controlToggle.textContent = 'Take Control';
        controlToggle.onclick = () => {
            this.playerControlled = !this.playerControlled;
            controlToggle.textContent = this.playerControlled ? 'Watch AI' : 'Take Control';
            const statusElement = document.getElementById('status');
            if (statusElement) {
                statusElement.textContent = this.playerControlled ? 'Player Controlled' : 'AI Controlled';
            }
            
            // Inform server about control mode change
            if (this.socket && this.socket.readyState === WebSocket.OPEN) {
                this.socket.send(JSON.stringify({ 
                    controlMode: this.playerControlled 
                }));
            }
        };
        controlPanel.appendChild(controlToggle);
        
        // Speed control
        const speedLabel = document.createElement('label');
        speedLabel.textContent = 'Game Speed: ';
        const speedSlider = document.createElement('input');
        speedSlider.type = 'range';
        speedSlider.min = '0.5';
        speedSlider.max = '2';
        speedSlider.step = '0.1';
        speedSlider.value = '0.7';
        speedSlider.oninput = () => {
            this.gameSpeed = parseFloat(speedSlider.value);
            if (speedValue) {
                speedValue.textContent = this.gameSpeed.toFixed(1) + 'x';
            }
            
            // Send game speed update to server
            if (this.socket && this.socket.readyState === WebSocket.OPEN) {
                this.socket.send(JSON.stringify({ 
                    gameSpeed: this.gameSpeed
                }));
            }
        };
        const speedValue = document.createElement('span');
        speedValue.textContent = '0.7x';
        
        speedLabel.appendChild(speedSlider);
        speedLabel.appendChild(speedValue);
        controlPanel.appendChild(speedLabel);
        
        // Toggle grid lines
        const gridToggle = document.createElement('button');
        gridToggle.textContent = 'Hide Grid';
        gridToggle.onclick = () => {
            this.showGrid = !this.showGrid;
            gridToggle.textContent = this.showGrid ? 'Hide Grid' : 'Show Grid';
        };
        controlPanel.appendChild(gridToggle);
        
        // Help text
        const helpText = document.createElement('div');
        helpText.className = 'help-text';
        helpText.innerHTML = 'Use <strong>Arrow Keys</strong> or <strong>WASD</strong> to navigate the maze when in player control mode';
        document.body.appendChild(helpText);
    }
    
    setupControls(): void {
        // Keyboard controls
        document.addEventListener('keydown', (e: KeyboardEvent) => {
            if (!this.playerControlled || this.gameState.game_over) return;
            
            let action: string | undefined;
            switch(e.code) {
                case 'ArrowUp':
                case 'KeyW':
                    action = 'up';
                    break;
                case 'ArrowDown':
                case 'KeyS':
                    action = 'down';
                    break;
                case 'ArrowLeft':
                case 'KeyA':
                    action = 'left';
                    break;
                case 'ArrowRight':
                case 'KeyD':
                    action = 'right';
                    break;
                default:
                    return;
            }
            
            this.movePlayer(action);
            e.preventDefault();
        });
    }
    
    movePlayer(direction: string): void {
        // Send movement command to server if in player mode
        if (this.socket && this.socket.readyState === WebSocket.OPEN) {
            this.socket.send(JSON.stringify({ action: direction }));
        }
    }
    
    connectWebSocket(): void {
        // Determine WebSocket URL based on the current page URL
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        
        this.socket = new WebSocket(wsUrl);
        
        this.socket.onopen = () => {
            console.log('WebSocket connection established');
            const statusElement = document.getElementById('status');
            if (statusElement) {
                statusElement.textContent = 'Connected (AI Controlled)';
            }
            
            // Request initial game settings
            if (this.socket) {
                this.socket.send(JSON.stringify({ action: 'getSettings' }));
            }
        };
        
        this.socket.onmessage = (event: MessageEvent) => {
            try {
                const data = JSON.parse(event.data);
                
                // Check if this is a settings response
                if (data.settings) {
                    this.updateSettingsUI(data.settings);
                } else {
                    // Regular game state update
                    this.gameState = data;
                    const scoreElement = document.getElementById('score');
                    if (scoreElement) {
                        scoreElement.textContent = this.gameState.score.toString();
                    }
                    
                    const statusElement = document.getElementById('status');
                    if (this.gameState.game_over) {
                        const status = this.gameState.won ? 
                            (this.playerControlled ? 'You Win!' : 'AI Wins!') : 
                            'Game Over';
                        if (statusElement) {
                            statusElement.textContent = status;
                        }
                    } else if (this.playerControlled) {
                        if (statusElement) {
                            statusElement.textContent = 'Player Controlled';
                        }
                    } else {
                        if (statusElement) {
                            statusElement.textContent = 'AI Controlled';
                        }
                    }
                }
            } catch (e) {
                console.error('Error parsing WebSocket message:', e);
            }
        };
        
        this.socket.onclose = () => {
            console.log('WebSocket connection closed');
            const statusElement = document.getElementById('status');
            if (statusElement) {
                statusElement.textContent = 'Disconnected';
            }
            
            // Try to reconnect after a delay
            setTimeout(() => this.connectWebSocket(), 2000);
        };
        
        this.socket.onerror = (error: Event) => {
            console.error('WebSocket error:', error);
        };
    }
    
    // Update UI elements with settings from the server
    updateSettingsUI(settings: any): void {
        if (settings.gameSpeed !== undefined) {
            const speedSlider = document.querySelector('input[type="range"][min="0.5"][max="2"]') as HTMLInputElement | null;
            if (speedSlider) {
                speedSlider.value = settings.gameSpeed;
                if (speedSlider.nextElementSibling) {
                    speedSlider.nextElementSibling.textContent = settings.gameSpeed.toFixed(1) + 'x';
                }
            }
        }
    }
    
    render(): void {
        // Clear canvas
        this.ctx.fillStyle = this.colors.background;
        this.ctx.fillRect(0, 0, this.width, this.height);
        
        // Calculate cell size to fit the canvas
        const cellWidth = this.width / this.gameState.maze.width;
        const cellHeight = this.height / this.gameState.maze.height;
        
        // Draw maze
        this.drawMaze(cellWidth, cellHeight);
        
        // Draw game stats
        this.drawStats();
        
        // Draw game over message if applicable
        if (this.gameState.game_over) {
            this.drawGameOver();
        }
        
        // Request next frame with game speed control
        setTimeout(() => {
            requestAnimationFrame(() => this.render());
        }, 1000/20 * (1/this.gameSpeed)); // Lower FPS for maze game
    }
    
    drawMaze(cellWidth: number, cellHeight: number): void {
        const maze = this.gameState.maze;
        
        for (let y = 0; y < maze.height; y++) {
            for (let x = 0; x < maze.width; x++) {
                const cell = maze.cells[y][x];
                
                // Calculate cell position
                const xPos = x * cellWidth;
                const yPos = y * cellHeight;
                
                // Fill cell based on type
                switch(cell) {
                    case this.CELL_TYPES.WALL:
                        this.ctx.fillStyle = this.colors.wall;
                        break;
                    case this.CELL_TYPES.PLAYER:
                        this.ctx.fillStyle = this.colors.player;
                        break;
                    case this.CELL_TYPES.GOAL:
                        this.ctx.fillStyle = this.colors.goal;
                        break;
                    default: // EMPTY
                        this.ctx.fillStyle = this.colors.empty;
                        break;
                }
                
                this.ctx.fillRect(xPos, yPos, cellWidth, cellHeight);
                
                // Draw grid lines if enabled
                if (this.showGrid) {
                    this.ctx.strokeStyle = '#cccccc';
                    this.ctx.strokeRect(xPos, yPos, cellWidth, cellHeight);
                }
            }
        }
        
        // Draw player with a nicer appearance
        const xPos = this.gameState.player_x * cellWidth;
        const yPos = this.gameState.player_y * cellHeight;
        
        // Draw a circle for the player
        this.ctx.fillStyle = this.colors.player;
        this.ctx.beginPath();
        this.ctx.arc(
            xPos + cellWidth/2, 
            yPos + cellHeight/2, 
            Math.min(cellWidth, cellHeight) * 0.4, 
            0, 
            Math.PI * 2
        );
        this.ctx.fill();
        
        // Draw goal indicator
        const goalX = this.gameState.goal_x * cellWidth;
        const goalY = this.gameState.goal_y * cellHeight;
        
        // Draw target/flag for goal
        this.ctx.fillStyle = this.colors.goal;
        this.ctx.beginPath();
        
        // Draw a flag or X pattern for the goal
        this.ctx.fillRect(
            goalX + cellWidth * 0.2, 
            goalY + cellHeight * 0.2, 
            cellWidth * 0.6, 
            cellHeight * 0.6
        );
        
        // Add an "X" pattern
        this.ctx.strokeStyle = 'white';
        this.ctx.lineWidth = 2;
        this.ctx.beginPath();
        this.ctx.moveTo(goalX + cellWidth * 0.2, goalY + cellHeight * 0.2);
        this.ctx.lineTo(goalX + cellWidth * 0.8, goalY + cellHeight * 0.8);
        this.ctx.moveTo(goalX + cellWidth * 0.8, goalY + cellHeight * 0.2);
        this.ctx.lineTo(goalX + cellWidth * 0.2, goalY + cellHeight * 0.8);
        this.ctx.stroke();
    }
    
    drawStats(): void {
        this.ctx.font = '16px Arial';
        this.ctx.fillStyle = 'black';
        this.ctx.textAlign = 'left';
        this.ctx.fillText(`Score: ${this.gameState.score}`, 10, 25);
    }
    
    drawGameOver(): void {
        // Semi-transparent overlay
        this.ctx.fillStyle = 'rgba(0, 0, 0, 0.5)';
        this.ctx.fillRect(0, 0, this.width, this.height);
        
        // Game result message
        this.ctx.font = 'bold 40px Arial';
        this.ctx.fillStyle = 'white';
        this.ctx.textAlign = 'center';
        
        const message = this.gameState.won ? 'MAZE SOLVED!' : 'GAME OVER';
        this.ctx.fillText(message, this.width / 2, this.height / 2 - 20);
        
        this.ctx.font = '20px Arial';
        this.ctx.fillText('New maze will generate shortly...', this.width / 2, this.height / 2 + 30);
    }
}

// Initialize the game when the page loads
window.addEventListener('DOMContentLoaded', () => {
    new MazeGameRenderer();
});
