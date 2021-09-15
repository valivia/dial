export abstract class dialModule {
  constructor(config: moduleConfig) {
    Object.assign(this, config);
  }

  public code: string;
  public name: string;
  public secondary = false;

  abstract run(argument?: number): Promise<void>
}

export type moduleConfig = {
  code: string;
  name: string;
  secondary: boolean;
}