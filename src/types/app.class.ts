import { dialModule } from "./dialModule.class.";

export class App {
  modules!: Map<string, dialModule>

  constructor() {
    this.modules = new Map();
  }

  getmodule(key: string): undefined | dialModule {
    return this.modules.get(key);
  }
}