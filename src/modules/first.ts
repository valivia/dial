import { dialModule } from "../types/dialModule.class.";

module.exports = class extends dialModule {
  constructor() {
    super({
      name: "test",
      code: "111",
      disabled: false,
    });
  }

  async run() {
    console.log("test");
    return true;
  }
};