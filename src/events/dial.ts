import colors from "colors";
import app from "../services/app.service";
colors.enable();

let currentNumber = 0;

let final = "";

let timeout: NodeJS.Timeout;

export default (): void => {
  currentNumber = (currentNumber + 1) % 10;

  if (timeout) clearTimeout(timeout);

  timeout = setTimeout(() => finalise(), 300);
};

function finalise() {
  final += `${currentNumber}`;
  console.log(`current final: ${final}`.green);

  if (final.length === 3) {
    runCommand(final);
    final = "";
  }

  currentNumber = 0;
}

function runCommand(id: string) {
  const module = app.getmodule(id);
  if (!module) {
    console.log("module not found.".red);
    return;
  }

  console.log(` * detected module: ${module.name} - ${module.code}`.green.italic);

  module?.run();
}