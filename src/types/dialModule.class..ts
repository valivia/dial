export abstract class dialModule {
  constructor(config: moduleConfig) {
    Object.assign(this, config);
  }

  public code: string;
  public name: string;
  public disabled? = false;

  abstract run(): Promise<boolean>
}

export type moduleConfig = {
  code: string;
  name: string;
  disabled: boolean;
}