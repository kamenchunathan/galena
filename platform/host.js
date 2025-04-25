import { readFileSync } from "fs";

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

  // Ensure values fit in 32 bits
  if (ptrBigInt > 0xffffffffn || lenBigInt > 0xffffffffn) {
    throw new Error("Pointer or length exceeds 32-bit maximum");
  }

  // Pack: lower 32 bits = pointer, upper 32 bits = length
  return (lenBigInt << 32n) | ptrBigInt;
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

      /**
       * Writes data from scatter/gather buffers to a file descriptor.
       *
       * @param {number} fd - The file descriptor (1 for stdout, 2 for stderr).
       * @param {number} iovs_ptr - Pointer to the array of iovec structures in Wasm memory.
       * @param {number} iovs_len - The number of iovec structures.
       * @param {number} nwritten_ptr - Pointer in Wasm memory to write the number of bytes written.
       * @returns {number} WASI errno code (0 for success).
       */

      fd_write: (fd, iovs_ptr, iovs_len, nwritten_ptr) => {
        // Get direct access to the Wasm memory buffer
        const memoryBuffer = memory.buffer;
        // Create a DataView for reading/writing multi-byte values (WASI uses little-endian)
        const view = new DataView(memoryBuffer);
        // Use TextDecoder to convert bytes to strings for logging
        const decoder = new TextDecoder();

        let totalBytesWritten = 0;
        const iovec_size = 8; // Each iovec is two u32s (ptr + len)

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
              return 8; // __WASI_ERRNO_IO (or EFAULT potentially)
            }

            // Get a view of the data buffer within the Wasm memory
            const dataBufferView = new Uint8Array(
              memoryBuffer,
              buf_ptr,
              buf_len,
            );

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
            // --- End Logging ---

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
      },
    },
    env: {
      js_read: function (ptr, size) {
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

  // console.log(greetPerson(wasm_module.instance, "Nathan Kamenchu"));

  initAndLogView(wasm_module.instance);
  updateAndLogView(wasm_module.instance, "wow");
  // updateAndLogView(wasm_module.instance, "Boom");
  // updateAndLogView(wasm_module.instance, "Wow");
  // updateAndLogView(wasm_module.instance, "lorem ipsum kdl alpwielfj wl");
  // try {
  //   wasm.instance.exports._start();
  // } catch (e) {
  //   const is_ok = e.message === "unreachable" && exit_code === 0;
  //   if (!is_ok) {
  //     console.error(e);
  //   }
  // }
}
function initAndLogView(instance) {
  const memory = instance.exports.memory;

  instance.exports.init(0);

  const res = instance.exports.view();
  const { ptr, len } = unpack_slice(res);

  const processedBytes = new Uint8Array(memory.buffer, ptr, len);
  const processedString = new TextDecoder().decode(processedBytes);
  console.log(`View: "${processedString}"`);
}

function updateAndLogView(instance, name) {
  const memory = instance.exports.memory;
  const inputString = name;

  const input_bytes = new TextEncoder().encode(inputString);
  const input_ptr = instance.exports.js_alloc_bytes(input_bytes.length);
  const view = new Uint8Array(memory.buffer, input_ptr, input_bytes.length);
  view.set(input_bytes);

  instance.exports.update(pack_slice(input_ptr, input_bytes.length));

  const res = instance.exports.view();
  const { ptr, len } = unpack_slice(res);

  // console.log(ptr, len);
  const processedBytes = new Uint8Array(memory.buffer, ptr, len);
  // console.log(processedBytes);
  const processedString = new TextDecoder().decode(processedBytes);
  console.log(`View: "${processedString}"`);
}

await roc_web_platform_run("./out/hello_world.wasm");
