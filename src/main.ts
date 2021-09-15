import { registermodules } from "./initializers/module.initializers";
import { App } from "./types/app.class";
import gpio, { Gpio } from "onoff";

import colors from "colors";
import app from "./services/app.service";
import dial from "./events/dial";
colors.enable();

export default class Main {
  private app: App;
  private dial: Gpio;

  constructor() {
    this.app = app;

    this.initialiseModules();
    this.initialiseGpio();
  }

  private initialiseModules() {
    registermodules(this.app).catch((e) => { throw ` x Couldnt load modules \n ${e}`.red.bold; });
  }

  private initialiseGpio() {
    const pins = gpio.Gpio;
    if (!pins.accessible) return;
    this.dial = new pins(17, "in", "rising", { debounceTimeout: 2 });

    this.dial.watch(() => dial());

    process.on("SIGINT", _ => {
      console.log("exiting..".red);
      this.dial.unexport();
    });
  }

  public getApp(): App {
    return this.app;
  }
}