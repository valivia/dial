import colors from "colors";
import app from "../services/app.service";
import { dialModule } from "../types/dialModule.class.";
colors.enable();

let currentNumber = 0;

let final = "";

let timeout: NodeJS.Timeout;

let selectedModule: dialModule | undefined;

export default (): void => {
  currentNumber = (currentNumber + 1) % 10;

  if (timeout) clearTimeout(timeout);

  timeout = setTimeout(() => finalise(), 300);
};

function finalise() {
  final += `${currentNumber}`;
  console.log(`current final: ${final}`.green);

  if ((final.length === 2) || (selectedModule && final.length === 1)) {
    runCommand(final);
    final = "";
  }

  currentNumber = 0;
}

function runCommand(id: string) {
  if (selectedModule) {
    selectedModule.run(Number(id));
    selectedModule = undefined;

    return;
  }

  selectedModule = app.getmodule(id);

  if (!selectedModule) {
    console.log("selectedModule not found.".red);
    return;
  }

  if (selectedModule.secondary) return;

  console.log(` * detected selectedModule: ${selectedModule.name} - ${selectedModule.code}`.green.italic);

  selectedModule?.run();
  selectedModule = undefined;
}