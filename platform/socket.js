import { TextEncoder, TextDecoder } from "util"; // Use Node's util if in Node.js env
// If in browser, these are globally available

// Assuming pack_slice and unpack_slice are defined as in your previous example
function unpack_slice(value) {
  const valueBigInt = BigInt(value);
  const lowBits = valueBigInt & 0xffffffffn;
  const highBits = (valueBigInt / 0x100000000n) | 0n;
  return {
    ptr: Number(lowBits), // Lower 32 bits as pointer
    len: Number(highBits), // Upper 32 bits as length
  };
}

function pack_slice(ptr, len) {
  const ptrBigInt = BigInt(ptr);
  const lenBigInt = BigInt(len);
  if (ptrBigInt > 0xffffffffn || lenBigInt > 0xffffffffn) {
    throw new Error("Pointer or length exceeds 32-bit maximum");
  }
  return (lenBigInt << 32n) | ptrBigInt;
}

class ReconnectingWebSocketConnector {
  /**
   * @param {string} wsUrl The URL of the WebSocket server.
   * @param {WebAssembly.Instance} wasmInstance The instantiated Wasm module.
   * @param {object} options Configuration options.
   * @param {string} options.wasmReceiveFunction Name of the exported Wasm function to call with incoming WS messages.
   * @param {string} options.wasmAllocFunction Name of the exported Wasm function to allocate memory.
   * @param {string} options.jsSendFunction Name of the *imported* JS function Wasm will call to send messages.
   * @param {number} [options.reconnectInterval=2000] Initial interval (ms) for reconnection attempts.
   * @param {number} [options.maxReconnectAttempts=10] Maximum number of consecutive reconnect attempts. Set to Infinity for unlimited.
   * @param {number} [options.maxBufferSize=100] Maximum number of messages to buffer while offline.
   * @param {number} [options.backoffFactor=1.5] Multiplier for reconnection interval increase (exponential backoff).
   */
  constructor(
    wsUrl,
    wasmInstance,
    {
      wasmReceiveFunction,
      wasmAllocFunction,
      jsSendFunction,
      reconnectInterval = 2000, // Initial reconnect delay 2s
      maxReconnectAttempts = 10, // Max attempts before giving up (can be Infinity)
      maxBufferSize = 100, // Max messages to queue
      backoffFactor = 1.5, // Exponential backoff factor
    },
  ) {
    if (
      !wsUrl ||
      !wasmInstance ||
      !wasmReceiveFunction ||
      !wasmAllocFunction ||
      !jsSendFunction
    ) {
      throw new Error(
        "Missing required constructor arguments for WasmWebSocketConnector",
      );
    }

    this.wsUrl = wsUrl;
    this.wasmInstance = wasmInstance;
    this.memory = wasmInstance.exports.memory;
    this.wasmReceiveFunction = wasmReceiveFunction;
    this.wasmAllocFunction = wasmAllocFunction;
    this.jsSendFunctionName = jsSendFunction;

    // Configuration
    this.reconnectInterval = reconnectInterval;
    this.maxReconnectAttempts = maxReconnectAttempts;
    this.maxBufferSize = maxBufferSize;
    this.backoffFactor = backoffFactor;

    // State
    this.websocket = null;
    this.isConnected = false;
    this.isConnecting = false;
    this.reconnectAttempts = 0;
    this.reconnectTimer = null;
    this.intentionalDisconnect = false;
    this.sendBuffer = []; // Stores { ptr, len, data } for buffered messages

    this.textEncoder = new TextEncoder();
    this.textDecoder = new TextDecoder();

    // Bind methods
    this.sendMessageToWebSocket = this.sendMessageToWebSocket.bind(this);
    this._handleOpen = this._handleOpen.bind(this);
    this._handleMessage = this._handleMessage.bind(this);
    this._handleError = this._handleError.bind(this);
    this._handleClose = this._handleClose.bind(this);
    this._attemptReconnect = this._attemptReconnect.bind(this);

    console.log(`Connector initialized. Wasm functions:
      Receive: ${this.wasmReceiveFunction}
      Allocate: ${this.wasmAllocFunction}
      JS Send Import: ${this.jsSendFunctionName}
      Reconnect Interval: ${this.reconnectInterval}ms, Max Attempts: ${this.maxReconnectAttempts}, Buffer Size: ${this.maxBufferSize}`);
  }

  /**
   * Initiates the WebSocket connection.
   */
  connect() {
    if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
      console.log("WebSocket already connected.");
      return;
    }
    if (this.isConnecting) {
      console.log("WebSocket connection attempt already in progress.");
      return;
    }

    this.intentionalDisconnect = false; // Reset flag on new connect attempt
    this.isConnecting = true;
    console.log(
      `Connecting to WebSocket at ${this.wsUrl}... (Attempt ${this.reconnectAttempts + 1})`,
    );

    try {
      // Ensure WebSocket API is available
      if (typeof WebSocket === "undefined") {
        throw new Error("WebSocket API not available in this environment.");
      }
      this.websocket = new WebSocket(this.wsUrl);
      this.websocket.binaryType = "arraybuffer";

      // Assign event handlers
      this.websocket.onopen = this._handleOpen;
      this.websocket.onmessage = this._handleMessage;
      this.websocket.onerror = this._handleError;
      this.websocket.onclose = this._handleClose;
    } catch (error) {
      console.error("Failed to create or connect WebSocket:", error);
      this.isConnecting = false;
      // If creation failed, schedule a reconnect attempt
      if (!this.intentionalDisconnect) {
        this._scheduleReconnect();
      }
    }
  }

  _handleOpen(event) {
    console.log("WebSocket connection established.");
    this.isConnected = true;
    this.isConnecting = false;
    // Reset attempts on successful connection
    this.reconnectAttempts = 0;

    // Clear  pending reconnect timer
    clearTimeout(this.reconnectTimer);
    this.reconnectTimer = null;

    // Process any buffered messages
    this._processSendBuffer();
  }

  _handleMessage(event) {
    this.forwardMessageToWasm(event.data);
  }

  _handleError(event) {
    console.error("WebSocket error observed:", event);
    this.isConnecting = false; // Ensure we are not stuck in connecting state on error
  }

  _handleClose(event) {
    console.log(
      `WebSocket connection closed. Code: ${event.code}, Reason: "${event.reason}", Was Clean: ${event.wasClean}`,
    );
    this.isConnected = false;
    this.isConnecting = false;
    this.websocket = null;
  }

  _scheduleReconnect() {
    const delay = 500;
    clearTimeout(this.reconnectTimer);
    this.reconnectTimer = setTimeout(this._attemptReconnect, delay);
  }

  _attemptReconnect() {
    if (!this.isConnected && !this.isConnecting) {
      this.connect();
    }
  }

  forwardMessageToWasm(data) {
    let inputBytes;
    let dataType = typeof data;

    if (data instanceof ArrayBuffer) {
      inputBytes = new Uint8Array(data);
      dataType = "ArrayBuffer";
    } else if (typeof data === "string") {
      inputBytes = this.textEncoder.encode(data);
    } else {
      console.warn(
        `Received unhandled WebSocket message type: ${typeof data}. Ignoring.`,
      );
      return;
    }

    try {
      const wasmFn = this.wasmInstance.exports[this.wasmReceiveFunction];
      const allocFn = this.wasmInstance.exports[this.wasmAllocFunction];

      if (!wasmFn || !allocFn) {
        console.error(
          `Wasm export not found for forwarding: ${!wasmFn ? this.wasmReceiveFunction : ""
          } ${!allocFn ? this.wasmAllocFunction : ""}`,
        );
        return;
      }

      const inputSize = inputBytes.length;

      // 1. Allocate memory in Wasm
      const inputPtr = allocFn(inputSize);
      if (typeof inputPtr !== "number" || inputPtr === 0) {
        console.error(
          `${this.wasmAllocFunction} returned invalid pointer: ${inputPtr} for size ${inputSize}`,
        );
        return;
      }

      // 2. Copy data to Wasm memory
      const wasmMemoryView = new Uint8Array(
        this.memory.buffer,
        inputPtr,
        inputSize,
      );
      wasmMemoryView.set(inputBytes);

      // 3. Pack pointer and length
      const packedSlice = pack_slice(inputPtr, inputSize);

      // 4. Call the Wasm function
      wasmFn(packedSlice); // Pass the packed slice

      // Assume Wasm deallocates the memory at inputPtr
    } catch (error) {
      console.error("Error forwarding message to Wasm:", error);
      // Consider if Wasm needs notification of the failure
    }
  }

  sendMessageToWebSocket(packedSlice) {
    try {
      const { ptr, len } = unpack_slice(packedSlice);

      if (ptr === 0 && len > 0) {
        console.error(
          `sendMessageToWebSocket received zero pointer with non-zero length (${len}). Invalid arguments from Wasm.`,
        );
        // Cannot read the memory safely. Wasm might need to deallocate ptr=0? Unlikely.
        return;
      }
      if (ptr !== 0 && len === 0) {
        console.warn(
          "sendMessageToWebSocket received zero length. Sending empty message.",
        );
        // Proceed to send empty message if connected, or buffer if not.
      }
      // Allow ptr=0, len=0 ? Maybe, depends on protocol. Let's treat as empty message for now.

      // Read the data FROM Wasm memory *immediately* to copy it.
      // This prevents issues if Wasm reuses or deallocates the memory before sending.
      let messageData;
      if (len > 0) {
        // Ensure memory bounds are checked BEFORE reading
        if (ptr + len > this.memory.buffer.byteLength) {
          console.error(
            `sendMessageToWebSocket error: Wasm provided buffer out of bounds (ptr=${ptr}, len=${len}, memory=${this.memory.buffer.byteLength}).`,
          );
          // Cannot read the memory safely. Wasm might need to deallocate ptr/len.
          return;
        }
        const wasmMemoryView = new Uint8Array(this.memory.buffer, ptr, len);
        messageData = new Uint8Array(wasmMemoryView); // Create a copy
      } else {
        messageData = new Uint8Array(0); // Empty data for zero length
      }

      // Now, decide whether to send immediately or buffer
      if (this.isConnected && this.websocket) {
        console.log(
          `Sending message immediately from Wasm (ptr=${ptr}, len=${len}, size=${messageData.length})`,
        );
        this.websocket.send(messageData); // Send the copied data (binary)
      } else {
        // Buffer the message if not connected
        if (this.sendBuffer.length >= this.maxBufferSize) {
          console.error(
            `Send buffer full (${this.maxBufferSize}). Dropping message from Wasm (ptr=${ptr}, len=${len}).`,
          );
          // Optional: Notify Wasm that the message was dropped? (Requires Wasm changes)
        } else {
          console.log(
            `Buffering message from Wasm (ptr=${ptr}, len=${len}, size=${messageData.length}). Connection state: ${this.isConnected ? "connected" : "disconnected"}/${this.isConnecting ? "connecting" : "idle"}`,
          );
          // Store the *copied* data along with original ptr/len for reference
          this.sendBuffer.push({ ptr, len, data: messageData });
        }
      }

      // IMPORTANT: Wasm is still responsible for deallocating the *original*
      // memory at `ptr` with `len` after this JS function returns.
      // The JS side now holds a *copy* in its buffer if needed.
    } catch (error) {
      console.error("Error processing message from Wasm for sending:", error);
      // Wasm might need to deallocate ptr/len if an error occurred here.
    }
  }

  /**
   * Processes and sends messages stored in the buffer.
   */
  _processSendBuffer() {
    if (!this.isConnected || !this.websocket) {
      console.warn("_processSendBuffer called while not connected.");
      return;
    }

    console.log(
      `Processing send buffer (${this.sendBuffer.length} messages)...`,
    );
    while (this.sendBuffer.length > 0) {
      const message = this.sendBuffer.shift(); // Get the oldest message
      if (message) {
        try {
          console.log(
            `Sending buffered message (original ptr=${message.ptr}, len=${message.len}, size=${message.data.length})`,
          );
          this.websocket.send(message.data); // Send the copied data
        } catch (error) {
          console.error("Error sending buffered message:", error);
          // Decide if we should re-buffer it or drop it. Re-buffering at the front:
          this.sendBuffer.unshift(message);
          // If the error persists (e.g., connection dropped *during* send),
          // the 'onclose' handler will eventually trigger, stopping the processing.
          break; // Stop processing buffer on error
        }
      }
    }
    if (this.sendBuffer.length === 0) {
      console.log("Send buffer empty.");
    }
  }

  /**
   * Closes the WebSocket connection intentionally and cleans up.
   */
  disconnect() {
    console.log("Disconnecting WebSocket intentionally...");
    this.intentionalDisconnect = true;
    clearTimeout(this.reconnectTimer); // Stop any scheduled reconnection
    this.reconnectTimer = null;
    this.sendBuffer = []; // Clear the buffer on intentional disconnect

    if (this.websocket) {
      this.websocket.close(1000, "Client disconnected intentionally"); // Use code 1000 for normal closure
    }
    // State should be updated by the onclose handler
    this.websocket = null;
    this.isConnected = false;
    this.isConnecting = false;
    this.reconnectAttempts = 0;
  }
}
