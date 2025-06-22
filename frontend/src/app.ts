import { ReconnectingWebSocket } from "./ws";
import init, { run } from "./rocApp";

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
    // const importObject = {
    //   env: {
    //     sendToBackend: (slice: number) => this.sendToBackend(slice),
    //   },
    // };

    await init(fetch(wasmPath));
    run();
  }

  async initializeWs(wsUrl: string) {
    this.ws = new ReconnectingWebSocket(wsUrl, {
      onMessage: (ev) => this.handleIncomingWsMessage(ev),
    });
    this.ws?.connect();
  }

  handleIncomingWsMessage(ev: MessageEvent) {
    console.log(ev);
    //   let data: ArrayBuffer;
    //
    //   if (typeof ev.data == "string") {
    //     data = new TextEncoder().encode(ev.data);
    //   } else {
    //     data = ev.data;
    //   }
    //
    //   const inputPtr = this.wasmExports!.js_alloc(data.byteLength);
    //   const view = new Uint8Array(this.memory!.buffer, inputPtr, data.byteLength);
    //   view.set(new Uint8Array(data));
    //
    //   this.wasmExports?.handle_ws_message(packSlice(inputPtr, data.byteLength));
    //
    //   // Re-render the view after processing the message
    //   this.renderView();
  }

  // sendToBackend(slice: number) {
  //   const { ptr, len } = unpackSlice(slice);
  //
  //   if (!this.memory) {
  //     console.error(
  //       "Cannot send to backend: WebAssembly memory not initialized",
  //     );
  //     return false;
  //   }
  //
  //   try {
  //     const dataView = new Uint8Array(this.memory.buffer, ptr, len);
  //     const dataCopy = new Uint8Array(dataView);
  //     this.ws?.sendMessageToWebSocket(new TextDecoder().decode(dataCopy));
  //
  //     return true;
  //   } catch (error) {
  //     console.error("Error sending data to backend:", error);
  //     return false;
  //   }
  // }
}
