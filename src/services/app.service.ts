import { App } from "../types/app.class";

declare const global: NodeJS.Global & { app?: App };


const app: App = global.app || new App();

if (process.env.NODE_ENV === "development") global.app = app;


export default app;