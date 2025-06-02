import { ReconnectingWebSocket } from "./ws";
import { packSlice, unpackSlice } from "./util";
import { renderViewToDOM } from "./dom";

interface HostExports extends WebAssembly.Exports {
  handle_ws_message: (packedSlice: BigInt) => null;
  js_alloc: (byteSize: number) => number;
  view: () => number;
  handle_dom_event: (callbackId: BigInt, valueSlice: BigInt) => void;
}

export class Application {
  wasm: WebAssembly.WebAssemblyInstantiatedSource | null = null;
  memory: WebAssembly.Memory | null = null;
  wasmExports: HostExports | null = null;
  ws: ReconnectingWebSocket | null = null;
  rootElement: HTMLElement | null = null;

  constructor(rootElementId: string = "app") {
    this.rootElement = document.getElementById(rootElementId);
    if (!this.rootElement) {
      console.error(`Root element with ID "${rootElementId}" not found`);
      this.rootElement = document.body;
    }
  }

  async initializeWasmModule(wasmPath: string) {
    const fetchPromise = fetch(wasmPath);
    const importObject = {
      wasi_snapshot_preview1: {
        proc_exit: procExit,
        random_get: (bufPtr: number, bufLen: number) =>
          randomGet(this.memory)(bufPtr, bufLen),
        fd_write: (
          fd: number,
          iovs_ptr: number,
          iovs_len: number,
          nwritten_ptr: number,
        ) => fdWrite(this.memory)(fd, iovs_ptr, iovs_len, nwritten_ptr),
      },

      env: {
        sendToBackend: (slice: number) => this.sendToBackend(slice),
      },
    };

    if (WebAssembly.instantiateStreaming) {
      this.wasm = await WebAssembly.instantiateStreaming(
        fetchPromise,
        importObject,
      );
    } else {
      const response = await fetchPromise;
      const moduleBuf = await response.arrayBuffer();
      this.wasm = await WebAssembly.instantiate(moduleBuf, importObject);
    }
    this.memory = this.wasm.instance.exports.memory as WebAssembly.Memory;
    this.wasmExports = this.wasm.instance.exports as HostExports;

    // TODO: Find a permanent home for calling init
    const init = this.wasm.instance.exports["init"] as () => null;
    init();

    // Render the initial view
    this.renderView();
  }

  renderView() {
    if (!this.wasmExports || !this.memory || !this.rootElement) {
      console.error("Cannot render view: Required components not initialized");
      return;
    }

    try {
      const viewSlice = this.wasmExports.view();
      const { ptr, len } = unpackSlice(viewSlice);

      const dataView = new Uint8Array(this.memory.buffer, ptr, len);
      const jsonString = new TextDecoder().decode(dataView);

      const viewJson = JSON.parse(jsonString);

      if (this.wasm?.instance.exports.handle_dom_event) {
        renderViewToDOM(
          this.rootElement,
          viewJson,
          (callbackId: number, value: string) => {
            console.debug("callback id: ", callbackId);
            const valueData = new TextEncoder().encode(value);
            const valuePtr = this.wasmExports!.js_alloc(valueData.byteLength);
            const valueView = new Uint8Array(
              this.memory!.buffer,
              valuePtr,
              valueData.byteLength,
            );
            valueView.set(valueData);

            // Call the WASM function with the packed slices
            this.wasmExports!.handle_dom_event(
              BigInt(callbackId),
              packSlice(valuePtr, valueData.byteLength),
            );
            this.renderView();
          },
        );
      }
    } catch (error) {
      console.error("Error rendering view:", error);
    }
  }

  async initializeWs(wsUrl: string) {
    this.ws = new ReconnectingWebSocket(wsUrl, {
      onMessage: (ev) => this.handleIncomingWsMessage(ev),
    });
    this.ws?.connect();
  }

  handleIncomingWsMessage(ev: MessageEvent) {
    let data: ArrayBuffer;

    if (typeof ev.data == "string") {
      data = new TextEncoder().encode(ev.data);
    } else {
      data = ev.data;
    }

    const inputPtr = this.wasmExports!.js_alloc(data.byteLength);
    const view = new Uint8Array(this.memory!.buffer, inputPtr, data.byteLength);
    view.set(new Uint8Array(data));

    this.wasmExports?.handle_ws_message(packSlice(inputPtr, data.byteLength));

    // Re-render the view after processing the message
    this.renderView();
  }

  sendToBackend(slice: number) {
    const { ptr, len } = unpackSlice(slice);

    if (!this.memory) {
      console.error(
        "Cannot send to backend: WebAssembly memory not initialized",
      );
      return false;
    }

    try {
      const dataView = new Uint8Array(this.memory.buffer, ptr, len);
      const dataCopy = new Uint8Array(dataView);
      this.ws?.sendMessageToWebSocket(new TextDecoder().decode(dataCopy));

      return true;
    } catch (error) {
      console.error("Error sending data to backend:", error);
      return false;
    }
  }
}

//-----------------------------------------------------------------------------------------------
//      WASI preview methods and environment functions provided to wasm module
//-----------------------------------------------------------------------------------------------

/// called by Wasm to terminate the process
function procExit(exitCode: number) {
  throw new Error(`Wasm proc_exit called with code ${exitCode}`);
}

/// Called by Wasm to fill a buffer with random bytes.
function randomGet(memory: WebAssembly.Memory | null) {
  return function (bufPtr: number, bufLen: number) {
    if (!memory) {
      console.error("WASI random_get called before memory was initialized!");
      return 5;
    }

    try {
      const bufferView = new Uint8Array(memory.buffer, bufPtr, bufLen);

      if (typeof crypto !== "undefined" && crypto.getRandomValues) {
        crypto.getRandomValues(bufferView);
      } else {
        console.error(
          "No secure random number generator available for random_get!",
        );

        // Return ENOEXEC (Exec format error) or similar
        return 8;
      }

      // (WASI errno __WASI_ERRNO_SUCCESS)
      return 0;
    } catch (error) {
      console.error("Error in random_get:", error);
      // Return EFAULT (Bad address) as a guess
      return 21;
    }
  };
}

function fdWrite(memory: WebAssembly.Memory | null) {
  return function (
    fd: number,
    iovs_ptr: number,
    iovs_len: number,
    nwritten_ptr: number,
  ) {
    if (!memory) {
      console.error("WASI random_get called before memory was initialized!");
      return 5;
    }

    const memoryBuffer = memory.buffer;
    const view = new DataView(memoryBuffer);
    const decoder = new TextDecoder();

    let totalBytesWritten = 0;
    const iovec_size = 8;

    try {
      for (let i = 0; i < iovs_len; i++) {
        const current_iovec_ptr = iovs_ptr + i * iovec_size;

        // Read buf_ptr (u32) and buf_len (u32) from the iovec struct
        // WASI uses little-endian, hence the 'true' argument
        const buf_ptr = view.getUint32(current_iovec_ptr, true);
        const buf_len = view.getUint32(current_iovec_ptr + 4, true);

        if (buf_ptr + buf_len > memoryBuffer.byteLength) {
          console.error(`fd_write error: iov[${i}] buffer out of bounds.`);
          // Write 0 bytes written back before erroring
          view.setUint32(nwritten_ptr, totalBytesWritten, true);

          // __WASI_ERRNO_IO (or EFAULT potentially)
          return 8;
        }

        // Get a view of the data buffer within the Wasm memory
        const dataBufferView = new Uint8Array(memoryBuffer, buf_ptr, buf_len);

        try {
          const text = decoder.decode(dataBufferView);
          console.log(text);
        } catch (e) {
          console.log(
            `  LOG (fd=${fd}): [Non-UTF8 data: ${Array.from(dataBufferView)
              .map((b) => b.toString(16).padStart(2, "0"))
              .join("")}]`,
          );
        }

        totalBytesWritten += buf_len;
      }

      // Write the total number of bytes written back to the specified pointer
      view.setUint32(nwritten_ptr, totalBytesWritten, true);

      return 0; // __WASI_ERRNO_SUCCESS
    } catch (e) {
      console.error("Error during fd_write (JS):", e);
      // Attempt to write 0 bytes written back in case of error
      try {
        view.setUint32(nwritten_ptr, 0, true);
      } catch (writeError) {
        console.error("Failed to write nwritten on error:", writeError);
      }

      return 8; // __WASI_ERRNO_IO (or another appropriate error)
    }
  };
}
