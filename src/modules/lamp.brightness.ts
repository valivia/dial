import { dialModule } from "../types/dialModule.class.";
import homeAssistantHttp from "../services/ha.service";

module.exports = class extends dialModule {
  constructor() {
    super({
      name: "lamp-brightness",
      code: "13",
      secondary: true,
    });
  }

  async run(value: number) {
    const brightness = Math.floor(((((value || 10) - 1) * (256 - 3)) / (10 - 1))) + 3;
    console.log(brightness);
    await homeAssistantHttp("post", "services/light/turn_on", { "device_id": "e8a39b8a282e31822a3d991d2e26ee9f", "brightness": brightness });
  }
};