interface MazeCell {
  readonly cells: number[][];
  readonly width: number;
  readonly height: number;
}

interface GameStateData {
  readonly maze: MazeCell;
  readonly player_x: number;
  readonly player_y: number;
  readonly goal_x: number;
  readonly goal_y: number;
  readonly score: number;
  readonly game_over: boolean;
  readonly won: boolean;
}

const CELL_TYPES = {
  EMPTY: 0,
  WALL: 1,
  PLAYER: 2,
  GOAL: 3,
} as const;

type CellType = (typeof CELL_TYPES)[keyof typeof CELL_TYPES];

interface ColorScheme {
  readonly background: string;
  readonly wall: string;
  readonly player: string;
  readonly goal: string;
  readonly empty: string;
  readonly text: string;
}

interface ControlModeMessage {
  readonly controlMode: boolean;
}

interface GameSpeedMessage {
  readonly gameSpeed: number;
}

interface ActionMessage {
  readonly action: "up" | "down" | "left" | "right" | "getSettings";
}

type WebSocketMessage = ControlModeMessage | GameSpeedMessage | ActionMessage;

interface SettingsResponse {
  readonly settings: {
    readonly gameSpeed: number;
  };
}

function debugLog(message: string, data?: any): void {
  console.log(`[DEBUG] ${message}`, data || '');
  const errorLog = document.getElementById('error-log');
  if (errorLog) {
    errorLog.style.display = 'block';
    errorLog.textContent += `[DEBUG] ${message} ${data ? JSON.stringify(data) : ''}\n`;
  }
}

class MazeGameRenderer {
  private readonly canvas: HTMLCanvasElement;
  private readonly ctx: CanvasRenderingContext2D;
  private readonly width: number;
  private readonly height: number;

  private readonly colors: ColorScheme;

  private gameState: GameStateData;
  private playerControlled: boolean;
  private showGrid: boolean;
  private gameSpeed: number;

  private socket: WebSocket | null;
  private initialized: boolean = false;

  constructor() {
    debugLog("MazeGameRenderer constructor started");
    
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

    const defaultWidth = 20;
    const defaultHeight = 15;
    const defaultCells = Array(defaultHeight).fill(null).map(() => 
      Array(defaultWidth).fill(CELL_TYPES.EMPTY)
    );
    
    for (let x = 0; x < defaultWidth; x++) {
      defaultCells[0][x] = CELL_TYPES.WALL;
      defaultCells[defaultHeight-1][x] = CELL_TYPES.WALL;
    }
    for (let y = 0; y < defaultHeight; y++) {
      defaultCells[y][0] = CELL_TYPES.WALL;
      defaultCells[y][defaultWidth-1] = CELL_TYPES.WALL;
    }
    
    this.gameState = {
      maze: {
        cells: defaultCells,
        width: defaultWidth, 
        height: defaultHeight,
      },
      player_x: 1,
      player_y: 1,
      goal_x: defaultWidth - 2,
      goal_y: defaultHeight - 2,
      score: 0,
      game_over: false,
      won: false,
    };
    
    this.gameState.maze.cells[1][1] = CELL_TYPES.PLAYER;
    this.gameState.maze.cells[defaultHeight-2][defaultWidth-2] = CELL_TYPES.GOAL;

    this.colors = {
      background: "#ffffff",
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

    debugLog("Attempting to connect WebSocket");
    this.connectWebSocket();
    
    debugLog("Setting up controls");
    this.setupControls();
    
    debugLog("Creating UI");
    this.createUI();
    
    debugLog("Starting render loop");
    this.render();

    debugLog("MazeGameRenderer constructor completed");
  }

  private createUI(): void {
    const controlPanel = document.createElement("div");
    controlPanel.className = "control-panel";
    document.body.appendChild(controlPanel);

    const controlToggle = document.createElement("button");
    controlToggle.textContent = "Take Control";
    controlToggle.onclick = (): void => {
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
        const message: ControlModeMessage = {
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

    speedSlider.oninput = (): void => {
      this.gameSpeed = parseFloat(speedSlider.value);
      if (speedValue) {
        speedValue.textContent = this.gameSpeed.toFixed(1) + "x";
      }

      if (this.socket && this.socket.readyState === WebSocket.OPEN) {
        const message: GameSpeedMessage = {
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
    gridToggle.onclick = (): void => {
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

  private setupControls(): void {
    document.addEventListener("keydown", (e: KeyboardEvent): void => {
      if (!this.playerControlled || this.gameState.game_over) return;

      let action: "up" | "down" | "left" | "right" | undefined;
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

  private movePlayer(direction: "up" | "down" | "left" | "right"): void {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      const message: ActionMessage = { action: direction };
      this.socket.send(JSON.stringify(message));
    }
  }

  private connectWebSocket(): void {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${protocol}//${window.location.host}/ws`;
    
    debugLog(`WebSocket URL: ${wsUrl}`);

    try {
      this.socket = new WebSocket(wsUrl);
      
      this.socket.onopen = (): void => {
        debugLog("WebSocket connection established");
        const statusElement = document.getElementById("status");
        if (statusElement) {
          statusElement.textContent = "Connected (AI Controlled)";
        }

        if (this.socket) {
          const message: ActionMessage = { action: "getSettings" };
          this.socket.send(JSON.stringify(message));
        }
      };

      this.socket.onmessage = (event: MessageEvent): void => {
        try {
          debugLog("WebSocket message received", event.data.substring(0, 100) + "...");
          const data = JSON.parse(event.data);

          if ("settings" in data) {
            const settings = data as SettingsResponse;
            this.updateSettingsUI(settings.settings);
          } else {
            if (!this.validateGameState(data)) {
              debugLog("Invalid game state received", data);
              return;
            }
            
            if (!this.initialized) {
              this.initialized = true;
              debugLog("Game initialized with valid data");
            }
            
            this.gameState = data as GameStateData;
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
            } else if (this.playerControlled) {
              if (statusElement) {
                statusElement.textContent = "Player Controlled";
              }
            } else {
              if (statusElement) {
                statusElement.textContent = "AI Controlled";
              }
            }
          }
        } catch (e) {
          debugLog(`Error parsing WebSocket message: ${(e as Error).message}`);
        }
      };

      this.socket.onclose = (): void => {
        debugLog("WebSocket connection closed");
        const statusElement = document.getElementById("status");
        if (statusElement) {
          statusElement.textContent = "Disconnected";
        }

        setTimeout(() => this.connectWebSocket(), 2000);
      };

      this.socket.onerror = (error: Event): void => {
        debugLog(`WebSocket error: ${JSON.stringify(error)}`);
      };
    } catch (e) {
      debugLog(`Error connecting to WebSocket: ${(e as Error).message}`);
    }
  }
  
  private validateGameState(data: any): boolean {
    if (!data.maze || !data.maze.cells || !Array.isArray(data.maze.cells)) {
      debugLog("Missing maze cells array in game state", data);
      return false;
    }
    
    if (data.maze.cells.length === 0) {
      debugLog("Empty maze cells array", data);
      return false;
    }
    
    for (let i = 0; i < data.maze.cells.length; i++) {
      if (!Array.isArray(data.maze.cells[i])) {
        debugLog(`Maze cells row ${i} is not an array`, data.maze.cells[i]);
        return false;
      }
    }
    
    return true;
  }

  private updateSettingsUI(settings: { readonly gameSpeed: number }): void {
    if (settings.gameSpeed !== undefined) {
      const speedSlider = document.querySelector(
        'input[type="range"][min="0.5"][max="2"]'
      ) as HTMLInputElement | null;
      
      if (speedSlider) {
        speedSlider.value = settings.gameSpeed.toString();
        const nextElement = speedSlider.nextElementSibling;
        if (nextElement) {
          nextElement.textContent = settings.gameSpeed.toFixed(1) + "x";
        }
      }
    }
  }

  private render(): void {
    try {
      this.ctx.fillStyle = this.colors.background;
      this.ctx.fillRect(0, 0, this.width, this.height);

      if (this.gameState && this.gameState.maze && this.gameState.maze.cells) {
        const cellWidth = this.width / this.gameState.maze.width;
        const cellHeight = this.height / this.gameState.maze.height;

        this.drawMaze(cellWidth, cellHeight);
        this.drawStats();

        if (this.gameState.game_over) {
          this.drawGameOver();
        }
      } else {
        this.ctx.fillStyle = "black";
        this.ctx.font = "20px Arial";
        this.ctx.textAlign = "center";
        this.ctx.fillText("Loading maze data...", this.width / 2, this.height / 2);
      }

      setTimeout(() => {
        requestAnimationFrame(() => this.render());
      }, (1000 / 20) * (1 / this.gameSpeed));
    } catch (e) {
      debugLog(`Render error: ${(e as Error).message}`);
    }
  }

  private drawMaze(cellWidth: number, cellHeight: number): void {
    const maze = this.gameState.maze;

    for (let y = 0; y < maze.height; y++) {
      for (let x = 0; x < maze.width; x++) {
        if (!maze.cells[y] || maze.cells[y][x] === undefined) {
          debugLog(`Missing cell at [${y}][${x}]`);
          continue;
        }

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
    this.ctx.arc(
      xPos + cellWidth / 2,
      yPos + cellHeight / 2,
      Math.min(cellWidth, cellHeight) * 0.4,
      0,
      Math.PI * 2
    );
    this.ctx.fill();

    const goalX = this.gameState.goal_x * cellWidth;
    const goalY = this.gameState.goal_y * cellHeight;

    this.ctx.fillStyle = this.colors.goal;
    this.ctx.beginPath();

    this.ctx.fillRect(
      goalX + cellWidth * 0.2,
      goalY + cellHeight * 0.2,
      cellWidth * 0.6,
      cellHeight * 0.6
    );

    this.ctx.strokeStyle = "white";
    this.ctx.lineWidth = 2;
    this.ctx.beginPath();
    this.ctx.moveTo(goalX + cellWidth * 0.2, goalY + cellHeight * 0.2);
    this.ctx.lineTo(goalX + cellWidth * 0.8, goalY + cellHeight * 0.8);
    this.ctx.moveTo(goalX + cellWidth * 0.8, goalY + cellHeight * 0.2);
    this.ctx.lineTo(goalX + cellWidth * 0.2, goalY + cellHeight * 0.8);
    this.ctx.stroke();
  }

  private drawStats(): void {
    this.ctx.font = "16px Arial";
    this.ctx.fillStyle = "black";
    this.ctx.textAlign = "left";
    this.ctx.fillText(`Score: ${this.gameState.score}`, 10, 25);
  }

  private drawGameOver(): void {
    this.ctx.fillStyle = "rgba(0, 0, 0, 0.5)";
    this.ctx.fillRect(0, 0, this.width, this.height);

    this.ctx.font = "bold 40px Arial";
    this.ctx.fillStyle = "white";
    this.ctx.textAlign = "center";

    const message = this.gameState.won ? "MAZE SOLVED!" : "GAME OVER";
    this.ctx.fillText(message, this.width / 2, this.height / 2 - 20);

    this.ctx.font = "20px Arial";
    this.ctx.fillText(
      "New maze will generate shortly...",
      this.width / 2,
      this.height / 2 + 30
    );
  }
}

window.addEventListener("DOMContentLoaded", () => {
  debugLog("DOM content loaded, initializing MazeGameRenderer");
  try {
    const game = new MazeGameRenderer();
    debugLog("MazeGameRenderer initialized successfully");
  } catch (e) {
    debugLog(`Failed to initialize MazeGameRenderer: ${(e as Error).message}`);
  }
});
