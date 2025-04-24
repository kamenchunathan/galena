import { readFileSync } from "fs";

function unpackSlice(value) {
  const valueBigInt = BigInt(value);
  const lowBits = valueBigInt & 0xffffffffn;
  const highBits = (valueBigInt / 0x100000000n) | 0n;

  return {
    ptr: Number(lowBits), // Lower 32 bits as pointer
    len: Number(highBits), // Upper 32 bits as length
  };
}

async function roc_web_platform_run(wasm_filename) {
  let wasm_module;
  let memory;

  const import_object = {
    wasi_snapshot_preview1: {
      /**
       * proc_exit: Called by Wasm to terminate the process.
       * @param {number} exitCode - The exit code provided by the Wasm module.
       */
      proc_exit: (exitCode) => {
        console.log(`Wasm module exited with code: ${exitCode}`);
        throw new Error(`Wasm proc_exit called with code ${exitCode}`);
      },

      /**
       * random_get: Called by Wasm to fill a buffer with random bytes.
       * @param {number} bufPtr - Pointer (byte offset) to the buffer in Wasm linear memory.
       * @param {number} bufLen - The length of the buffer in bytes.
       * @returns {number} errno - 0 on success, non-zero on error (WASI errno code).
       */
      random_get: (bufPtr, bufLen) => {
        if (!memory) {
          console.error(
            "WASI random_get called before memory was initialized!",
          );
          return 5;
        }
        try {
          const bufferView = new Uint8Array(memory.buffer, bufPtr, bufLen);

          if (typeof crypto !== "undefined" && crypto.getRandomValues) {
            crypto.getRandomValues(bufferView);
          } else if (typeof require === "function") {
            // Fallback for older Node.js versions (less secure)
            const nodeCrypto = require("crypto");
            const randomBytes = nodeCrypto.randomBytes(bufLen);
            bufferView.set(randomBytes);
          } else {
            console.error(
              "No secure random number generator available for random_get!",
            );
            return 8; // Return ENOEXEC (Exec format error) or similar
          }

          return 0; // Return 0 for success (WASI errno __WASI_ERRNO_SUCCESS)
        } catch (error) {
          console.error("Error in random_get:", error);
          return 21; // Return EFAULT (Bad address) as a guess
        }
      },
    },
    env: {
      js_read: function(ptr, size) {
        const processedBytes = new Uint8Array(memory.buffer, ptr, size);
        read_buf = new TextDecoder().decode(processedBytes);
      },
      roc_panic: (_pointer, _tag_id) => {
        throw "Roc panicked!";
      },
      roc_dbg: (_loc, _msg) => {
        // TODO write a proper impl.
        throw "Roc dbg not supported!";
      },
    },
  };

  const fetchPromise = fetch(wasm_filename);
  try {
    if (WebAssembly.instantiateStreaming) {
      // streaming API has better performance if available
      // It can start compiling Wasm before it has fetched all of the bytes, so we don't `await` the request!
      wasm_module = await WebAssembly.instantiateStreaming(
        fetchPromise,
        import_object,
      );
    } else {
      const response = await fetchPromise;
      const module_bytes = await response.arrayBuffer();
      wasm_module = await WebAssembly.instantiate(module_bytes, import_object);
    }
  } catch {
    const module_bytes = readFileSync(wasm_filename);
    wasm_module = await WebAssembly.instantiate(module_bytes, import_object);
  }
  memory = wasm_module.instance.exports.memory;

  console.log(greetPerson(wasm_module.instance, "Nathan Kamenchu"));
  // try {
  //   wasm.instance.exports._start();
  // } catch (e) {
  //   const is_ok = e.message === "unreachable" && exit_code === 0;
  //   if (!is_ok) {
  //     console.error(e);
  //   }
  // }
}

function greetPerson(instance, name) {
  const memory = instance.exports.memory;
  const inputString = name;

  const input_bytes = new TextEncoder().encode(inputString);
  const input_ptr = instance.exports.js_alloc_bytes(input_bytes.length);
  const view = new Uint8Array(memory.buffer, input_ptr, input_bytes.length);
  view.set(input_bytes);

  const res = instance.exports.js_greet_person(input_ptr, input_bytes.length);

  const { ptr, len } = unpackSlice(res);
  const processedBytes = new Uint8Array(memory.buffer, ptr, len);
  const processedString = new TextDecoder().decode(processedBytes);
  console.log(`Read processed data back: "${processedString}"`);
}

await roc_web_platform_run("./out/hello_world.wasm");
