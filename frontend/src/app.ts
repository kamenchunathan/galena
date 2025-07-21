import init, { run } from "./rocApp";

export class Application {
  constructor(rootElementId: string = "root") {
    if (!document.getElementById(rootElementId)) {
      console.error(`Root element with ID "${rootElementId}" not found`);
      document.body.id = rootElementId;
    }
  }

  async initializeWasmModule(wasmPath: string) {
    await init(fetch(wasmPath));
    run();
  }
}
