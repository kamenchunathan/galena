import { Application } from "./app";

function main() {
  const app = new Application("root");

  app.initializeWasmModule("/app.wasm");
}

main();
