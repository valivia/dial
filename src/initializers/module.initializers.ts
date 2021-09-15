import fs from "fs";
import path from "path";
import { App } from "../types/app.class";
import { dialModule } from "../types/dialModule.class.";

export async function registermodules(app: App): Promise<void> {
  console.log(" > Loading modules".green.bold);

  const files = fs.readdirSync(path.join(__dirname, "../modules"));

  for (const file of files) {
    const moduleClass = (await import(`../modules/${file}`)).default;
    const module = new moduleClass(app) as dialModule;

    if (module == undefined) { continue; }

    if (app.getmodule(module.code) !== undefined) {
      console.log(`duplicate modules with name: ${module.name}`.red.bold);
      process.exit();
    }

    // Add module to app.
    app.modules.set(module.code, module);
    // Log.
    console.log(` - Loaded module: ${module.name} - ${module.code}`.cyan.italic);
  }
  console.log(" ✓ All modules loaded".green.bold);
}