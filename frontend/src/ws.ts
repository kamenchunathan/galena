export class ReconnectingWebSocket {
  wsUrl: string;

  // Configuration
  reconnectInterval: number;
  maxReconnectAttempts: number;
  maxBufferSize: number;

  // State
  websocket: WebSocket | null;
  isConnected: boolean;
  isConnecting: boolean;
  reconnectAttempts: number;
  reconnectTimer: number | null;
  sendBuffer: (ArrayBuffer | string)[];

  // Callbacks
  onOpen: ((_: Event) => null) | null;
  onMessage: ((_: MessageEvent) => null) | null;
  onClose: (() => null) | null;

  constructor(
    wsUrl: string,
    onOpen = null,
    onMessage = null,
    onClose = null,
    reconnectInterval = 2000,
    maxReconnectAttempts = 200,
    maxBufferSize = 100,
  ) {
    this.wsUrl = wsUrl;

    // Configuration
    this.reconnectInterval = reconnectInterval;
    this.maxReconnectAttempts = maxReconnectAttempts;
    this.maxBufferSize = maxBufferSize;

    // State
    this.websocket = null;
    this.isConnected = false;
    this.isConnecting = false;
    this.reconnectAttempts = 0;
    this.reconnectTimer = null;
    this.sendBuffer = [];

    this.onOpen = onOpen;
    this.onMessage = onMessage;
    this.onClose = onClose;
  }

  connect() {
    if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
      console.log("WebSocket already connected.");
      return;
    }

    if (this.isConnecting) {
      console.log("WebSocket connection attempt already in progress.");
      return;
    }

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
      this._scheduleReconnect();
    }
  }

  _handleOpen(event: Event) {
    this.isConnected = true;
    this.isConnecting = false;
    this.reconnectAttempts = 0;

    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.reconnectTimer = null;

    // this._processSendBuffer();
    console.log(this._processSendBuffer);

    // Call callback
    if (this.onOpen) this.onOpen(event);
  }

  _handleMessage(event: MessageEvent) {
    if (this.onMessage) this.onMessage(event.data);
  }

  _handleError(_event: Event) {
    // Ensure we're not stuck in the connecting phase
    this.isConnecting = false;
  }

  _handleClose(event: CloseEvent) {
    console.log(
      `WebSocket connection closed. Code: ${event.code}, Reason: "${event.reason}", Was Clean: ${event.wasClean}`,
    );
    this.isConnected = false;
    this.isConnecting = false;
    this.websocket = null;

    if (this.onClose) this.onClose();
  }

  _scheduleReconnect() {
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.reconnectTimer = setTimeout(
      this._attemptReconnect,
      this.reconnectInterval,
    );
  }

  _attemptReconnect() {
    if (!this.isConnected && !this.isConnecting) {
      this.connect();
    }
  }

  sendMessageToWebSocket(msg: string) {
    if (this.isConnected && this.websocket) {
      this.websocket.send(msg);
    } else {
      if (this.sendBuffer.length >= this.maxBufferSize) {
        console.error(
          `Send buffer full (${this.maxBufferSize}). Dropping message ${msg}`,
        );
      } else {
        this.sendBuffer.push(msg);
      }
    }
  }

  _processSendBuffer() {
    if (!this.isConnected || !this.websocket) {
      console.warn("_processSendBuffer called while not connected.");
      return;
    }

    console.log(
      `Processing send buffer (${this.sendBuffer.length} messages)...`,
    );
    while (this.sendBuffer.length > 0) {
      const message = this.sendBuffer.shift();
      if (message) {
        try {
          this.websocket.send(message);
        } catch (error) {
          this.sendBuffer.unshift(message);
          break;
        }
      }
    }
  }
}
