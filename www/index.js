import { Game } from "snake-rs-wasm";
import { memory } from "snake-rs-wasm/snake_rs_wasm_bg";



let keyCode = '';
function handleKeyEvent(event) {
    //console.log("keypress event, code: " + event.code);
    keyCode = event.code;
    if (keyCode === 'Space') {
        togglePlay();
    }
}
function registerKeyEventHandlers(event) {
    console.log("DOMContentLoaded event");
    let body = document.getElementsByTagName("body")[0];
    body.addEventListener('keypress', handleKeyEvent);


    document.getElementById("button-up").addEventListener('click', e => {
        keyCode = 'KeyW';
    });
    document.getElementById("button-left").addEventListener('click', e => {
        keyCode = 'KeyA';
    });
    document.getElementById("button-down").addEventListener('click', e => {
        keyCode = 'KeyS'
    });
    document.getElementById("button-right").addEventListener('click', e => {
        keyCode = 'KeyD';
    });
}

if (document.readyState === 'loading') {  // Loading hasn't finished yet
    document.addEventListener('DOMContentLoaded', registerKeyEventHandlers);
} else {  // `DOMContentLoaded` has already fired
    registerKeyEventHandlers();
}

const textCanvas = document.getElementById("snake-text-canvas");

let game = Game.default();
let width = game.width();
let height = game.height();
let speed = game.speed();
let gameOver = false;

// init creation controls
document.getElementById('width').value = width;
document.getElementById('height').value = height;
document.getElementById('speed').value = speed;

document.getElementById('create').addEventListener("click", event => {
  let newWidth = document.getElementById('width').value;
  let newHeight = document.getElementById('height').value;
  let newSpeed = document.getElementById('speed').value;
  game = Game.new(newWidth, newHeight, newSpeed);
  width = game.width();
  height = game.height();
  speed = game.speed();

  document.getElementById('game-over').style.display = 'none';
  gameOver = false;
  play();
});


let animationId = null;

const renderLoop = () => {
  readGamepadButtons();
  let done = game.tick(keyCode);

  document.getElementById('score').textContent = game.score();
  document.getElementById('current-speed').textContent = game.speed();

  drawGame();

  if (!done) {
    animationId = requestAnimationFrame(renderLoop);
  } else {
    document.getElementById('game-over').style.display = 'block'
    gameOver = true;
  }
};


const drawGame = () => {
  game.draw();
  const screenBufferLen = game.screen_buffer_len();
  const screenBufferPtr = game.screen_buffer();
  const screenBuffer = new Uint16Array(memory.buffer, screenBufferPtr, screenBufferLen);
  const screenStr = String.fromCharCode.apply(null, screenBuffer);
  textCanvas.textContent = screenStr;
};


const isPaused = () => {
  return animationId === null;
};

const playPauseButton = document.getElementById("play-pause");

const play = () => {
  playPauseButton.textContent = "⏸";
  document.getElementById('pause').style.display = 'none'
  renderLoop();
};

const pause = () => {
  playPauseButton.textContent = "▶";
  document.getElementById('pause').style.display = 'block'
  cancelAnimationFrame(animationId);
  animationId = null;
};

const togglePlay = () => {
  if (!gameOver) {
    if (isPaused()) {
      play();
    } else {
      pause();
    }
  }
}

playPauseButton.addEventListener("click", event => {
    togglePlay();
});

//play();
pause();
drawGame();


///////////////////////////////////////////////////////////////////
// gamepad

let gamepadsElement = document.getElementById('gamepads');
let gamepadList = document.getElementById('gamepad-list');

let gamepads = [];

function gamepadConnectionHandler(event, connecting) {
  var gamepad = event.gamepad;
  if (connecting) {
    // log
    console.log("Gamepad connected at index %d: %s. %d buttons, %d axes.",
      gamepad.index, gamepad.id,
      gamepad.buttons.length, gamepad.axes.length);

    // store
    gamepads[gamepad.index] = gamepad;

    // display
    if (gamepads.length === 1) {
      gamepadsElement.style.display = 'block'
    }
    let li = document.createElement("li");
    li.textContent = gamepad.id;
    gamepadList.appendChild(li);
  } else {
    // log
    console.log("Gamepad disconnected from index %d: %s",
      gamepad.index, gamepad.id);

    // store
    gamepads.splice(gamepad.index, 1);

    // display
    if (gamepads.length === 0) {
      gamepadsElement.style.display = 'none'
    }
    for (let i=0; i<gamepadList.children.length; i++) {
      let li = gamepadList.children[i];
      if (li.textContent == gamepad.id) {
        gamepadList.removeChild(li);
      }
    }
  }
}

function buttonPressed(b) {
  if (typeof(b) == "object") {
    return b.pressed;
  }
  return b == 1.0;
}

function readGamepadButtons() {
    for (let i=0; i<gamepads.length; i++) {
        let gamepad = gamepads[i];
        for (let j=0; j<gamepad.buttons.length; j++) {
            let button = gamepad.buttons[j];
            if (buttonPressed(button)) {
                //console.log("button pressed %d (%s)", j, gamepad.id);
                // steam gamepad
                if (j === 5 || j === 17) {
                    keyCode = 'KeyW';
                }
                else if (j === 3 || j === 20) {
                    keyCode = 'KeyD';
                }
                else if (j === 2 || j === 0) {
                    keyCode = 'KeyS';
                }
                else if (j === 4 || j === 19) {
                    keyCode = 'KeyA';
                }
            }
        }
    }
}

window.addEventListener("gamepadconnected", (e) => {
  gamepadConnectionHandler(e, true);
});
window.addEventListener("gamepaddisconnected", (e) => {
  gamepadConnectionHandler(e, false);
});

//window.setInterval(readGamepadButtons, 500);
