class Application {
  wasm: WebAssembly.WebAssemblyInstantiatedSource;

  constructor() { }

  async initializeWasmModule(wasmPath: string) {
    const fetchPromise = fetch(wasmPath);
    const importObject = {};

    try {
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
    } catch {
      // TODO: This part is only for testing using node. Remove for browser code

      // @ts-ignore
      import { readFileSync } from "fs";

      const moduleBytes = readFileSync(wasmPath);
      this.wasm = await WebAssembly.instantiate(moduleBytes, importObject);
    }
  }
}
//-----------------------------------------------------------------------------------------------
//      WASI preview methods and environment functions provided to wasm module
//-----------------------------------------------------------------------------------------------

/// called by Wasm to terminate the process
function proc_exit(exitCode: number) {
  throw new Error(`Wasm proc_exit called with code ${exitCode}`);
}

/// Called by Wasm to fill a buffer with random bytes.

function randomGet(memory: WebAssembly.Memory) {
  return function(bufPtr: number, bufLen: number) {
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
