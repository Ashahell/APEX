import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export async function defaultHealthCheck(command: string): Promise<boolean> {
  try {
    await execAsync(command, { timeout: 5000 });
    return true;
  } catch {
    return false;
  }
}

export async function checkCommandExists(command: string): Promise<boolean> {
  return defaultHealthCheck(`${command} --version`);
}

export async function checkGit(): Promise<boolean> {
  return checkCommandExists('git');
}

export async function checkNode(): Promise<boolean> {
  return checkCommandExists('node');
}

export async function checkDocker(): Promise<boolean> {
  return checkCommandExists('docker');
}

export async function checkAws(): Promise<boolean> {
  return checkCommandExists('aws');
}

export async function checkKubectl(): Promise<boolean> {
  return checkCommandExists('kubectl');
}
