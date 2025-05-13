"use strict";
const CELL_TYPES = {
    EMPTY: 0,
    WALL: 1,
    PLAYER: 2,
    GOAL: 3,
};
class MazeGameRenderer {
    constructor() {
        const canvasElement = document.getElementById("game");
        if (!canvasElement || !(canvasElement instanceof HTMLCanvasElement)) {
            throw new Error("Canvas element not found");
        }
        this.canvas = canvasElement;
        const context = this.canvas.getContext("2d");
        if (!context) {
            throw new Error("Could not get 2D context");
        }
        this.ctx = context;
        this.width = this.canvas.width;
        this.height = this.canvas.height;
        this.gameState = {
            maze: {
                cells: [],
                width: 20,
                height: 15,
            },
            player_x: 1,
            player_y: 1,
            goal_x: 18,
            goal_y: 13,
            score: 0,
            game_over: false,
            won: false,
        };
        this.colors = {
            background: "#f0f0f0",
            wall: "#333333",
            player: "#4285F4",
            goal: "#EA4335",
            empty: "#FFFFFF",
            text: "#000000",
        };
        this.playerControlled = false;
        this.showGrid = true;
        this.gameSpeed = 0.7;
        this.socket = null;
        this.connectWebSocket();
        this.setupControls();
        this.render();
        this.createUI();
    }
    createUI() {
        const controlPanel = document.createElement("div");
        controlPanel.className = "control-panel";
        document.body.appendChild(controlPanel);
        const controlToggle = document.createElement("button");
        controlToggle.textContent = "Take Control";
        controlToggle.onclick = () => {
            this.playerControlled = !this.playerControlled;
            controlToggle.textContent = this.playerControlled
                ? "Watch AI"
                : "Take Control";
            const statusElement = document.getElementById("status");
            if (statusElement) {
                statusElement.textContent = this.playerControlled
                    ? "Player Controlled"
                    : "AI Controlled";
            }
            if (this.socket && this.socket.readyState === WebSocket.OPEN) {
                const message = {
                    controlMode: this.playerControlled,
                };
                this.socket.send(JSON.stringify(message));
            }
        };
        controlPanel.appendChild(controlToggle);
        const speedLabel = document.createElement("label");
        speedLabel.textContent = "Game Speed: ";
        const speedSlider = document.createElement("input");
        speedSlider.type = "range";
        speedSlider.min = "0.5";
        speedSlider.max = "2";
        speedSlider.step = "0.1";
        speedSlider.value = "0.7";
        const speedValue = document.createElement("span");
        speedValue.textContent = "0.7x";
        speedSlider.oninput = () => {
            this.gameSpeed = parseFloat(speedSlider.value);
            if (speedValue) {
                speedValue.textContent = this.gameSpeed.toFixed(1) + "x";
            }
            if (this.socket && this.socket.readyState === WebSocket.OPEN) {
                const message = {
                    gameSpeed: this.gameSpeed,
                };
                this.socket.send(JSON.stringify(message));
            }
        };
        speedLabel.appendChild(speedSlider);
        speedLabel.appendChild(speedValue);
        controlPanel.appendChild(speedLabel);
        const gridToggle = document.createElement("button");
        gridToggle.textContent = "Hide Grid";
        gridToggle.onclick = () => {
            this.showGrid = !this.showGrid;
            gridToggle.textContent = this.showGrid ? "Hide Grid" : "Show Grid";
        };
        controlPanel.appendChild(gridToggle);
        const helpText = document.createElement("div");
        helpText.className = "help-text";
        helpText.innerHTML =
            "Use <strong>Arrow Keys</strong> or <strong>WASD</strong> to navigate the maze when in player control mode";
        document.body.appendChild(helpText);
    }
    setupControls() {
        document.addEventListener("keydown", (e) => {
            if (!this.playerControlled || this.gameState.game_over)
                return;
            let action;
            switch (e.code) {
                case "ArrowUp":
                case "KeyW":
                    action = "up";
                    break;
                case "ArrowDown":
                case "KeyS":
                    action = "down";
                    break;
                case "ArrowLeft":
                case "KeyA":
                    action = "left";
                    break;
                case "ArrowRight":
                case "KeyD":
                    action = "right";
                    break;
                default:
                    return;
            }
            this.movePlayer(action);
            e.preventDefault();
        });
    }
    movePlayer(direction) {
        if (this.socket && this.socket.readyState === WebSocket.OPEN) {
            const message = { action: direction };
            this.socket.send(JSON.stringify(message));
        }
    }
    connectWebSocket() {
        const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        this.socket = new WebSocket(wsUrl);
        this.socket.onopen = () => {
            console.log("WebSocket connection established");
            const statusElement = document.getElementById("status");
            if (statusElement) {
                statusElement.textContent = "Connected (AI Controlled)";
            }
            if (this.socket) {
                const message = { action: "getSettings" };
                this.socket.send(JSON.stringify(message));
            }
        };
        this.socket.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                if ("settings" in data) {
                    const settings = data;
                    this.updateSettingsUI(settings.settings);
                }
                else {
                    this.gameState = data;
                    const scoreElement = document.getElementById("score");
                    if (scoreElement) {
                        scoreElement.textContent = this.gameState.score.toString();
                    }
                    const statusElement = document.getElementById("status");
                    if (this.gameState.game_over) {
                        const status = this.gameState.won
                            ? this.playerControlled
                                ? "You Win!"
                                : "AI Wins!"
                            : "Game Over";
                        if (statusElement) {
                            statusElement.textContent = status;
                        }
                    }
                    else if (this.playerControlled) {
                        if (statusElement) {
                            statusElement.textContent = "Player Controlled";
                        }
                    }
                    else {
                        if (statusElement) {
                            statusElement.textContent = "AI Controlled";
                        }
                    }
                }
            }
            catch (e) {
                console.error("Error parsing WebSocket message:", e);
            }
        };
        this.socket.onclose = () => {
            console.log("WebSocket connection closed");
            const statusElement = document.getElementById("status");
            if (statusElement) {
                statusElement.textContent = "Disconnected";
            }
            setTimeout(() => this.connectWebSocket(), 2000);
        };
        this.socket.onerror = (error) => {
            console.error("WebSocket error:", error);
        };
    }
    updateSettingsUI(settings) {
        if (settings.gameSpeed !== undefined) {
            const speedSlider = document.querySelector('input[type="range"][min="0.5"][max="2"]');
            if (speedSlider) {
                speedSlider.value = settings.gameSpeed.toString();
                const nextElement = speedSlider.nextElementSibling;
                if (nextElement) {
                    nextElement.textContent = settings.gameSpeed.toFixed(1) + "x";
                }
            }
        }
    }
    render() {
        this.ctx.fillStyle = this.colors.background;
        this.ctx.fillRect(0, 0, this.width, this.height);
        const cellWidth = this.width / this.gameState.maze.width;
        const cellHeight = this.height / this.gameState.maze.height;
        this.drawMaze(cellWidth, cellHeight);
        this.drawStats();
        if (this.gameState.game_over) {
            this.drawGameOver();
        }
        setTimeout(() => {
            requestAnimationFrame(() => this.render());
        }, (1000 / 20) * (1 / this.gameSpeed));
    }
    drawMaze(cellWidth, cellHeight) {
        const maze = this.gameState.maze;
        for (let y = 0; y < maze.height; y++) {
            for (let x = 0; x < maze.width; x++) {
                const cell = maze.cells[y][x];
                const xPos = x * cellWidth;
                const yPos = y * cellHeight;
                switch (cell) {
                    case CELL_TYPES.WALL:
                        this.ctx.fillStyle = this.colors.wall;
                        break;
                    case CELL_TYPES.PLAYER:
                        this.ctx.fillStyle = this.colors.player;
                        break;
                    case CELL_TYPES.GOAL:
                        this.ctx.fillStyle = this.colors.goal;
                        break;
                    default:
                        this.ctx.fillStyle = this.colors.empty;
                        break;
                }
                this.ctx.fillRect(xPos, yPos, cellWidth, cellHeight);
                if (this.showGrid) {
                    this.ctx.strokeStyle = "#cccccc";
                    this.ctx.strokeRect(xPos, yPos, cellWidth, cellHeight);
                }
            }
        }
        const xPos = this.gameState.player_x * cellWidth;
        const yPos = this.gameState.player_y * cellHeight;
        this.ctx.fillStyle = this.colors.player;
        this.ctx.beginPath();
        this.ctx.arc(xPos + cellWidth / 2, yPos + cellHeight / 2, Math.min(cellWidth, cellHeight) * 0.4, 0, Math.PI * 2);
        this.ctx.fill();
        const goalX = this.gameState.goal_x * cellWidth;
        const goalY = this.gameState.goal_y * cellHeight;
        this.ctx.fillStyle = this.colors.goal;
        this.ctx.beginPath();
        this.ctx.fillRect(goalX + cellWidth * 0.2, goalY + cellHeight * 0.2, cellWidth * 0.6, cellHeight * 0.6);
        this.ctx.strokeStyle = "white";
        this.ctx.lineWidth = 2;
        this.ctx.beginPath();
        this.ctx.moveTo(goalX + cellWidth * 0.2, goalY + cellHeight * 0.2);
        this.ctx.lineTo(goalX + cellWidth * 0.8, goalY + cellHeight * 0.8);
        this.ctx.moveTo(goalX + cellWidth * 0.8, goalY + cellHeight * 0.2);
        this.ctx.lineTo(goalX + cellWidth * 0.2, goalY + cellHeight * 0.8);
        this.ctx.stroke();
    }
    drawStats() {
        this.ctx.font = "16px Arial";
        this.ctx.fillStyle = "black";
        this.ctx.textAlign = "left";
        this.ctx.fillText(`Score: ${this.gameState.score}`, 10, 25);
    }
    drawGameOver() {
        this.ctx.fillStyle = "rgba(0, 0, 0, 0.5)";
        this.ctx.fillRect(0, 0, this.width, this.height);
        this.ctx.font = "bold 40px Arial";
        this.ctx.fillStyle = "white";
        this.ctx.textAlign = "center";
        const message = this.gameState.won ? "MAZE SOLVED!" : "GAME OVER";
        this.ctx.fillText(message, this.width / 2, this.height / 2 - 20);
        this.ctx.font = "20px Arial";
        this.ctx.fillText("New maze will generate shortly...", this.width / 2, this.height / 2 + 30);
    }
}
window.addEventListener("DOMContentLoaded", () => {
    console.log("MazeGameRenderer initializing...");
    try {
        const game = new MazeGameRenderer();
        console.log("MazeGameRenderer initialized successfully");
    }
    catch (e) {
        console.error("Failed to initialize MazeGameRenderer:", e);
        const errorDisplay = document.getElementById("error-log");
        if (errorDisplay) {
            errorDisplay.textContent +=
                "MazeGameRenderer Error: " + e.message;
            errorDisplay.style.display = "block";
        }
    }
});
