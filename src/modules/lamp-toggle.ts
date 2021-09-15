import { dialModule } from "../types/dialModule.class.";
import homeAssistantHttp from "../services/ha.service";

module.exports = class extends dialModule {
  constructor() {
    super({
      name: "lamp-toggle",
      code: "11",
      secondary: false,
    });
  }

  async run() {
    const services = await homeAssistantHttp("post", "services/light/toggle", { "device_id": "e8a39b8a282e31822a3d991d2e26ee9f" });

    console.log(services);
  }
};