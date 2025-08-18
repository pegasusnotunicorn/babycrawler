import initTurbo, * as turbo from "./pkg/turbo_genesis_impl_wasm_bindgen.js";

/**************************************************/
/* BOOTLOADER IMAGE URLS                          */
/**************************************************/

const TURBO_LOGO_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAGAAAAASCAYAAACkctvyAAAAAXNSR0IArs4c6QAAARxJREFUWIXdWcsOxCAIrI3//8vsycbYzcLAiGTnjIOUx2DargkiIlcArbWmcc02ms9hi95r9WH1h3J5eFc+V4C/yC0fdIZmn1EUKM+MKOftPbwb1so7CUbh3lnV/w0s3xlY78q4u4hI2Q5goXqSe5Tg1KhAxJzNxUxq9wijx5HnnDVQix2TSwMi/OEO8CIaqHft83JZC+i1Zira+NIA74dhtmW1Dcha0Z57QyKcOX4qYaeQHxtBGqo8wKLQOqZbjf8VzG1KRARNfEoHIEFFx1XkPOtxhdibNSBjjlt8aDae1/gAci6KEQftJcxMEPNjsPd6Np4EnJ7/SJDsLkDASMbMYeqA3eOn8prKvtvKRxVh5AcKQ2x3VDqqQ9FYP+Ko6Ck9Nrm3AAAAAElFTkSuQmCC";
const LOADING_IMAGE_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACUAAAAFCAYAAADCMiU1AAAAAXNSR0IArs4c6QAAAIpJREFUKJGVkjEKAzEMBHchXe7gujQpUub/b8oDXLifNDowimUuCwJ5bFlrIwFoIkJXOIMyn+XV+tStMiRpP3PbXvHQLuk+4eO9zfax6idJU1OSZLtHQd76VOeLTx/1qoxFfRtd/jgHtggSf0ZkvgEPoI8858ABtLTXo/4N9OU8XJ2Rf2cqPyr3+wJvIvPpXApPMAAAAABJRU5ErkJggg==";
const CLICK_TO_PLAY_IMGAGE_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEQAAAAGCAYAAAB6gzjVAAAAAXNSR0IArs4c6QAAARtJREFUOI2VlDtuxDAMRB9ci8iWLrZIlRTJfXwAu3Plw/gshvYIOUEOkD4Gkp5ppAWt1cchIMDkcIajDwyAhsBEmudqLV5OI+We0fwPt9bf2gtAp6o7ICLCvu+pYG6ABoEqL+IFDQKGiFhNjZqtQxER+r5veq5oSuQevHrvFdgBt23ba/iO9YO4rdV43ntV1UvES5syei7Nc/MtL655nl9anu1K8DiH6LUDcM49qSpd130C76UNpFHiDcPA7Xb7XpblDfg9IXXvcc7ReBwAjOPINE0/67p+AM9neitzFbjcc3N6DviyNXOCDzdQ4sUBBstG7UYDr/hCAnYN68AteL62Xn0aOUAz9VJPWiv15+ae8XKGZ7klD8X/IsAf/YRX5UHT7BoAAAAASUVORK5CYII=";
const ERROR_IMAGE_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABsAAAAFCAYAAAC0CJe+AAAAAXNSR0IArs4c6QAAAGxJREFUKJGNjkEKgDAMBKfgzRZ8gA/o/1/Wg/f1EjW2Gg0UlumQbBIImwQJ4IlF3P9F/mS5ME4BZkFLsETclpYjd0VOH4EETdB8o4PJHYr4W775BqqgdtIq2AT5J8/2+mOXb0BRu6/sFg/M5x2RCGV7WoVq5QAAAABJRU5ErkJggg==";
const PLEASE_TRY_AGAIN_IMAGE_URL =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFUAAAAFCAYAAAA0e6CtAAAAAXNSR0IArs4c6QAAANxJREFUOI2lkl0OwyAMg0Ha21qpB+AAvf+ZdoBK6/u3hwYpi2LYDxJqMI5toBWg2Ki11l4D+HXHFDdiiq80fu2LObPcmfY/Z/Memf/N6rWUcgeOWuumAjluDLMmZkfkKg2x/0x4Wc61ryeeo6xkfYqf8N7vjWscNjePZyadK/px+Ba5SmNwAdOco7yj/CEr9v30bAROf9Brz8AGtFlI4+7AHrBm05stDn+MNNQFJN4NOIFlxleaIutpeXfg/ICfPUj/iZBhcGOGqTpyR3jmr9bqwUd6Mw2V9cu6AbwAVsYUIXubCLIAAAAASUVORK5CYII=";

/**************************************************/
/* GAMEPAD SUPPORT                                */
/**************************************************/

class GamepadManager {
  constructor(canvas) {
    this.canvas = canvas;
    this.gamepads = {};
    this.prevButtonStates = {};
    this.axisStates = {
      ArrowUp: false,
      ArrowDown: false,
      ArrowLeft: false,
      ArrowRight: false,
    };
    this.init();
  }

  init() {
    window.addEventListener("gamepadconnected", (e) =>
      this.onGamepadConnected(e)
    );
    window.addEventListener("gamepaddisconnected", (e) =>
      this.onGamepadDisconnected(e)
    );
    this.poll();
  }

  onGamepadConnected(event) {
    const gamepad = event.gamepad;
    console.log(
      `Gamepad connected at index ${gamepad.index}: ${gamepad.id}. ${gamepad.buttons.length} buttons, ${gamepad.axes.length} axes.`
    );
    this.gamepads[gamepad.index] = gamepad;
    this.prevButtonStates[gamepad.index] = gamepad.buttons.map(
      (button) => button.pressed
    );
  }

  onGamepadDisconnected(event) {
    const gamepad = event.gamepad;
    console.log(
      `Gamepad disconnected from index ${gamepad.index}: ${gamepad.id}`
    );
    delete this.gamepads[gamepad.index];
    delete this.prevButtonStates[gamepad.index];
  }

  poll() {
    const connectedGamepads = navigator.getGamepads
      ? navigator.getGamepads()
      : navigator.webkitGetGamepads
      ? navigator.webkitGetGamepads()
      : [];

    for (let gp of connectedGamepads) {
      if (gp) {
        if (!this.gamepads[gp.index]) {
          this.onGamepadConnected({ gamepad: gp });
        } else {
          this.updateGamepadState(gp);
        }
      }
    }

    requestAnimationFrame(() => this.poll());
  }

  updateGamepadState(gamepad) {
    const prevStates = this.prevButtonStates[gamepad.index];
    gamepad.buttons.forEach((button, index) => {
      if (button.pressed !== prevStates[index]) {
        if (button.pressed) {
          this.dispatchButtonEvent(gamepad, index, "keydown");
        } else {
          this.dispatchButtonEvent(gamepad, index, "keyup");
        }
        this.prevButtonStates[gamepad.index][index] = button.pressed;
      }
    });

    // Handle axes (e.g., left stick)
    this.handleAxes(gamepad);
  }

  dispatchButtonEvent(gamepad, buttonIndex, eventType) {
    let keyEvent;
    switch (buttonIndex) {
      case 0: // A
        keyEvent = new KeyboardEvent(eventType, { key: "z", code: "KeyZ" });
        break;
      case 1: // B
        keyEvent = new KeyboardEvent(eventType, { key: "x", code: "KeyX" });
        break;
      case 12: // D-pad Up
        keyEvent = new KeyboardEvent(eventType, {
          key: "ArrowUp",
          code: "ArrowUp",
        });
        break;
      case 13: // D-pad Down
        keyEvent = new KeyboardEvent(eventType, {
          key: "ArrowDown",
          code: "ArrowDown",
        });
        break;
      case 14: // D-pad Left
        keyEvent = new KeyboardEvent(eventType, {
          key: "ArrowLeft",
          code: "ArrowLeft",
        });
        break;
      case 15: // D-pad Right
        keyEvent = new KeyboardEvent(eventType, {
          key: "ArrowRight",
          code: "ArrowRight",
        });
        break;
      // Add more mappings as needed
      default:
        return; // Unmapped button
    }
    console.log(keyEvent);

    this.canvas.dispatchEvent(keyEvent);
  }

  handleAxes(gamepad) {
    const threshold = 0.5;
    // Example: Left Stick Horizontal (axes[0]), Vertical (axes[1])
    const x = gamepad.axes[0];
    const y = gamepad.axes[1];

    // Horizontal
    if (x > threshold) {
      if (!this.axisStates.ArrowRight) {
        this.dispatchAxisEvent("ArrowRight", "keydown");
        this.axisStates.ArrowRight = true;
      }
    } else {
      if (this.axisStates.ArrowRight) {
        this.dispatchAxisEvent("ArrowRight", "keyup");
        this.axisStates.ArrowRight = false;
      }
    }

    if (x < -threshold) {
      if (!this.axisStates.ArrowLeft) {
        this.dispatchAxisEvent("ArrowLeft", "keydown");
        this.axisStates.ArrowLeft = true;
      }
    } else {
      if (this.axisStates.ArrowLeft) {
        this.dispatchAxisEvent("ArrowLeft", "keyup");
        this.axisStates.ArrowLeft = false;
      }
    }

    // Vertical
    if (y > threshold) {
      if (!this.axisStates.ArrowDown) {
        this.dispatchAxisEvent("ArrowDown", "keydown");
        this.axisStates.ArrowDown = true;
      }
    } else {
      if (this.axisStates.ArrowDown) {
        this.dispatchAxisEvent("ArrowDown", "keyup");
        this.axisStates.ArrowDown = false;
      }
    }

    if (y < -threshold) {
      if (!this.axisStates.ArrowUp) {
        this.dispatchAxisEvent("ArrowUp", "keydown");
        this.axisStates.ArrowUp = true;
      }
    } else {
      if (this.axisStates.ArrowUp) {
        this.dispatchAxisEvent("ArrowUp", "keyup");
        this.axisStates.ArrowUp = false;
      }
    }
  }

  dispatchAxisEvent(key, eventType) {
    const event = new KeyboardEvent(eventType, { key: key, code: key });
    this.canvas.dispatchEvent(event);
  }
}

/**************************************************/
/* WASM IMPORT PROXY                              */
/**************************************************/

// This proxy prevents WebAssembly.LinkingError from being thrown
// prettier-ignore
window.createWasmImportsProxy = (target = {}) => {
  console.log(target);
  return new Proxy(target, {
    get: (target, namespace) => {
      // Stub each undefined namespace with a Proxy
      target[namespace] = target[namespace] ?? new Proxy({}, {
        get: (_, prop) => {
          // Generate a sub function for any accessed property
          return (...args) => {
            console.log(`Calling ${namespace}.${prop} with arguments:`, args);
            // Implement the actual function logic here
          };
        }
      });
      return target[namespace];
    }
  })
};

/**************************************************/
/* GLOBAL STUBS                                   */
/**************************************************/

window.turboSolUser = window.turboSolUser ?? (() => null);
window.turboSolGetAccount = window.turboSolGetAccount ?? (async () => {});
window.turboSolSignAndSendTransaction =
  window.turboSolSignAndSendTransaction ?? (async () => {});

/**************************************************/
/* TOUCH CONTROLS                                 */
/**************************************************/

function initializeNipple(canvas) {
  const presses = {
    up: {
      keydown: new KeyboardEvent("keydown", {
        key: "ArrowUp",
        code: "ArrowUp",
      }),
      keyup: new KeyboardEvent("keyup", {
        key: "ArrowUp",
        code: "ArrowUp",
      }),
    },
    down: {
      keydown: new KeyboardEvent("keydown", {
        key: "ArrowDown",
        code: "ArrowDown",
      }),
      keyup: new KeyboardEvent("keyup", {
        key: "ArrowDown",
        code: "ArrowDown",
      }),
    },
    left: {
      keydown: new KeyboardEvent("keydown", {
        key: "ArrowLeft",
        code: "ArrowLeft",
      }),
      keyup: new KeyboardEvent("keyup", {
        key: "ArrowLeft",
        code: "ArrowLeft",
      }),
    },
    right: {
      keydown: new KeyboardEvent("keydown", {
        key: "ArrowRight",
        code: "ArrowRight",
      }),
      keyup: new KeyboardEvent("keyup", {
        key: "ArrowRight",
        code: "ArrowRight",
      }),
    },
  };
  let active = null;
  nipplejs
    .create({ dataOnly: true })
    .on("dir:up", (e) => {
      if (active && active !== presses.up) {
        canvas.dispatchEvent(active.keyup);
      }
      canvas.dispatchEvent(presses.up.keydown);
      active = presses.up;
    })
    .on("dir:down", (e) => {
      if (active && active !== presses.down) {
        canvas.dispatchEvent(active.keyup);
      }
      canvas.dispatchEvent(presses.down.keydown);
      active = presses.down;
    })
    .on("dir:left", (e) => {
      if (active && active !== presses.left) {
        canvas.dispatchEvent(active.keyup);
      }
      canvas.dispatchEvent(presses.left.keydown);
      active = presses.left;
    })
    .on("dir:right", (e) => {
      if (active && active !== presses.right) {
        canvas.dispatchEvent(active.keyup);
      }
      canvas.dispatchEvent(presses.right.keydown);
      active = presses.right;
    })
    .on("end", (e) => {
      if (active) {
        canvas.dispatchEvent(active.keyup);
      }
      active = null;
    });
  // Disable double-tap zoom on mobile
  document.addEventListener("dblclick", (e) => e.preventDefault());
}

/**************************************************/
/* FETCH WITH PROGRESS UTIL                       */
/**************************************************/

async function fetchWithProgress(init, cb = () => {}) {
  const res = await fetch(init);
  const contentEncoding = res.headers.get("content-encoding");
  const contentLength =
    res.headers.get("content-length") ??
    res.headers.get("x-goog-stored-content-length");
  // If there's no content-length or if the response is encoded,
  // we can't trust preallocation.
  if (!contentLength || contentEncoding) {
    let loaded = 0;
    const chunks = [];
    const reader = res.body.getReader();
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      loaded += value.length;
      cb(loaded);
      chunks.push(value);
    }
    // Combine the chunks into one Uint8Array.
    const total = chunks.reduce((acc, chunk) => acc + chunk.length, 0);
    const body = new Uint8Array(total);
    let offset = 0;
    for (const chunk of chunks) {
      body.set(chunk, offset);
      offset += chunk.length;
      cb(offset, total);
    }
    return body;
  }
  // When content-length is available and there's no content-encoding,
  // preallocate the buffer.
  const total = parseInt(contentLength, 10);
  let loaded = 0;
  const body = new Uint8Array(new ArrayBuffer(total));
  const reader = res.body.getReader();
  while (true) {
    const { done, value: chunk } = await reader.read();
    if (done) break;
    body.set(chunk, loaded);
    loaded += chunk.length;
    cb(loaded, total);
  }
  return body;
}

/**************************************************/
/* WAIT FOR USER INTERACTION                      */
/**************************************************/

function waitForUserInteraction() {
  return new Promise((resolve) => {
    const events = ["click", "touchstart", "keydown"];
    const handler = (event) => {
      events.forEach((e) => window.removeEventListener(e, handler));
      resolve(event);
    };
    events.forEach((e) => window.addEventListener(e, handler));
  });
}

//**************************************************/
/* IMAGE UTILS                                     */
/***************************************************/

function loadImage(src) {
  return new Promise((resolve) => {
    const img = new Image();
    img.src = src;
    img.onload = () => resolve(img);
  });
}

function getAspectRatio(a, b) {
  return a / b;
}

function center(screen, item) {
  return Math.floor((screen - item) / 2);
}

//**************************************************/
/* RUN TURBO GAME                                  */
/***************************************************/

async function run() {
  // Initialize a temporary 2D context canvas for loading state.
  const loadingCanvas = document.createElement("canvas");
  const player = document.getElementById("player");
  player?.appendChild(loadingCanvas);
  const ctx = loadingCanvas.getContext("2d");

  // Simple function to draw loading bar progress
  function drawProgressBar(progress) {
    // Progress fill
    if (progress > 0) {
      ctx.fillStyle = "#007BFF";
      ctx.fillRect(barX, barY, Math.floor(barWidth * progress), barHeight);
    }
  }

  // Set canvas size
  loadingCanvas.width = 387;
  loadingCanvas.height = 387;
  const screenWidth = loadingCanvas.width;
  const screenHeight = loadingCanvas.height;

  // Update background color
  loadingCanvas.style.backgroundColor = "#000000";
  ctx.fillStyle = "#000000";
  ctx.fillRect(0, 0, screenWidth, screenHeight);

  // Draw TURBO logo
  const turboLogoImage = await loadImage(TURBO_LOGO_URL);
  const logoWidth = turboLogoImage.width;
  const logoHeight = turboLogoImage.height;
  const logoX = center(screenWidth, logoWidth);
  const logoY = center(screenHeight, logoHeight);
  ctx.drawImage(turboLogoImage, logoX, logoY, logoWidth, logoHeight);

  // Draw "LOADING" text
  const loadingImage = await loadImage(LOADING_IMAGE_URL);
  const loadingWidth = loadingImage.width;
  const loadingHeight = loadingImage.height;
  const loadingX = center(screenWidth, loadingWidth);
  const loadingY = logoY + 52;
  ctx.drawImage(loadingImage, loadingX, loadingY);

  // Prograss bar settings
  const barWidth = logoWidth;
  const barHeight = 2;
  const barX = center(screenWidth, barWidth);
  const barY = loadingY + loadingHeight + 8;
  let progress = 0;

  // Update progress bar to 0%
  drawProgressBar(progress / 100);

  // Download Turbo's WASM runtime.
  console.log("Loading runtime...");
  const runtime = await fetchWithProgress(
    "pkg/turbo_genesis_impl_wasm_bindgen_bg.wasm",
    (loaded, total) => {
      if (!total) return;
      // Update progress bar up to 0-50%
      progress = (loaded / total) * 50;
      drawProgressBar(progress / 100);
    }
  );

  // Update progress bar to 75%
  progress = 75;
  drawProgressBar(progress / 100);

  // Initalize Turbo's WASM runtime.
  console.log("Initializing runtime...");
  await initTurbo({
    module_or_path: runtime.buffer,
  });

  // Update progress bar to 80%
  progress = 80;
  drawProgressBar(progress / 100);

  // Fetch Turbo File.
  console.log("Loading game data...");
  const turbofile = await fetchWithProgress("main.turbo", (loaded, total) => {
    if (!total) return;
    // Update progress bar to 80-85%
    progress = Math.max(80, (loaded / total) * 85);
    drawProgressBar(progress / 100);
  });

  // Update progress bar to 90%
  progress = 90;
  drawProgressBar(progress / 100);

  // Decode Turbo File contents.
  console.log(`Decompressing game data...`);
  const contents = turbo.decode_turbofile_v0_contents(
    new Uint8Array(turbofile)
  );

  // Update progress bar to 93%
  progress = 93;
  drawProgressBar(progress / 100);

  // Initialize context.
  const canvas = document.createElement("canvas");
  canvas.width = 360;
  canvas.height = 640;

  // Initialize nipple (aka virtual analog stick).
  console.log("Initializing touch controls...");
  initializeNipple(canvas);

  // Update progress bar to 96%
  progress = 96;
  drawProgressBar(progress / 100);

  // Initialize Gamepad Support.
  console.log("Initializing gamepad...");
  const gamepadManager = new GamepadManager(canvas);
  gamepadManager.poll();

  // Update progress bar to 99%
  progress = 99;
  drawProgressBar(progress / 100);

  // Display prompt to click if game has audio.
  // User interaction is required to play back audio without errors.
  if (contents.has_audio()) {
    const clickToPlayImage = await loadImage(CLICK_TO_PLAY_IMGAGE_URL);
    const clickWidth = clickToPlayImage.width;
    const clickHeight = clickToPlayImage.height;
    const clickX = center(screenWidth, clickWidth);
    const clickY = logoY + 52;
    ctx.fillStyle = "#5200FF";
    ctx.fillRect(0, 0, screenWidth, screenHeight);
    ctx.drawImage(turboLogoImage, logoX, logoY, logoWidth, logoHeight);
    ctx.drawImage(clickToPlayImage, clickX, clickY, clickWidth, clickHeight);
    loadingCanvas.style.backgroundColor = "#5200FF";
    await waitForUserInteraction();
  }

  // Append game canvas.
  player?.removeChild(loadingCanvas);
  player?.appendChild(canvas);

  // Set up turboGameEvent listener.
  window.addEventListener("turboGameEvent", (e) => {
    console.log(e.detail);
  });

  // Run game.
  await turbo.run(canvas, contents);
}

try {
  await run();
} catch (err) {
  console.error("Turbo failed to initialize", err);

  // Clear the screen
  const loadingCanvas = document.querySelector("canvas");
  const ctx = loadingCanvas?.getContext("2d");
  if (ctx) {
    const screenWidth = loadingCanvas.width;
    const screenHeight = loadingCanvas.height;

    // Update background color
    ctx.fillStyle = "#000000";
    ctx.fillRect(0, 0, screenWidth, screenHeight);
    loadingCanvas.style.backgroundColor = "#000000";

    // Re-render logo
    const turboLogoImage = await loadImage(TURBO_LOGO_URL);
    const logoWidth = turboLogoImage.width;
    const logoHeight = turboLogoImage.height;
    const logoX = center(screenWidth, logoWidth);
    const logoY = center(screenHeight, logoHeight);
    ctx.drawImage(turboLogoImage, logoX, logoY, logoWidth, logoHeight);

    // Load images
    const [errorImage, pleaseTryAgainImage] = await Promise.all([
      loadImage(ERROR_IMAGE_URL),
      loadImage(PLEASE_TRY_AGAIN_IMAGE_URL),
    ]);

    // Draw "error" smaller and centered, just below logo
    const errorWidth = errorImage.width;
    const errorHeight = errorImage.height;
    const errorX = center(screenWidth, errorWidth);
    const errorY = logoY + logoHeight + 8;
    ctx.drawImage(errorImage, errorX, errorY, errorWidth, errorHeight);

    // Draw pleaseTryAgainImage smaller and centered, 85% of the way down
    const ptaHeight = pleaseTryAgainImage.height;
    const ptaWidth = pleaseTryAgainImage.width;
    const ptaX = center(screenWidth, ptaWidth);
    const ptaY = logoY + 52;
    ctx.drawImage(pleaseTryAgainImage, ptaX, ptaY, ptaWidth, ptaHeight);
  }
}
