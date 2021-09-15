import { dialModule } from "../types/dialModule.class.";
import homeAssistantHttp from "../services/ha.service";

module.exports = class extends dialModule {
  constructor() {
    super({
      name: "lamp-temp",
      code: "12",
      secondary: true,
    });
  }

  async run(value: number) {
    const mireds = Math.floor(((((value || 10) - 1) * (370 - 153)) / (10 - 1))) + 153;
    console.log(mireds);
    await homeAssistantHttp("post", "services/light/turn_on", { "device_id": "e8a39b8a282e31822a3d991d2e26ee9f", "color_temp": mireds });
  }
};