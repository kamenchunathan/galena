import { Application } from "./app";

function main() {
  const app = new Application();

  app.initializeWasmModule("/app.wasm");

  app.initializeWs("/ws");

  console.log("wow");
}

main();
