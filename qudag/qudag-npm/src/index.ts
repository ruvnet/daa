/**
 * QuDAG NPM Package Main Module
 * Provides programmatic access to QuDAG functionality
 */

import { spawn, SpawnOptions } from 'child_process';
import { ensureBinary, getBinaryPath, getPlatformInfo } from './binary-manager';

export { ensureBinary, getBinaryPath, getPlatformInfo };

/**
 * Execute a QuDAG command
 */
export async function execute(
  args: string[],
  options?: SpawnOptions
): Promise<{ code: number; stdout: string; stderr: string }> {
  // Ensure binary is available
  await ensureBinary();
  
  const binaryPath = getBinaryPath();
  
  return new Promise((resolve, reject) => {
    const stdout: string[] = [];
    const stderr: string[] = [];
    
    const child = spawn(binaryPath, args, {
      ...options,
      stdio: options?.stdio || 'pipe'
    });
    
    if (child.stdout) {
      child.stdout.on('data', (data) => {
        stdout.push(data.toString());
      });
    }
    
    if (child.stderr) {
      child.stderr.on('data', (data) => {
        stderr.push(data.toString());
      });
    }
    
    child.on('error', reject);
    
    child.on('exit', (code) => {
      resolve({
        code: code || 0,
        stdout: stdout.join(''),
        stderr: stderr.join('')
      });
    });
  });
}

/**
 * QuDAG CLI wrapper class for programmatic usage
 */
export class QuDAG {
  /**
   * Start a QuDAG node
   */
  static async start(port?: number): Promise<{ code: number; stdout: string; stderr: string }> {
    const args = ['start'];
    if (port) {
      args.push('--port', port.toString());
    }
    return execute(args);
  }
  
  /**
   * Stop the QuDAG node
   */
  static async stop(): Promise<{ code: number; stdout: string; stderr: string }> {
    return execute(['stop']);
  }
  
  /**
   * Get node status
   */
  static async status(): Promise<{ code: number; stdout: string; stderr: string }> {
    return execute(['status']);
  }
  
  /**
   * List peers
   */
  static async listPeers(): Promise<{ code: number; stdout: string; stderr: string }> {
    return execute(['peer', 'list']);
  }
  
  /**
   * Add a peer
   */
  static async addPeer(address: string): Promise<{ code: number; stdout: string; stderr: string }> {
    return execute(['peer', 'add', address]);
  }
  
  /**
   * Register a dark address
   */
  static async registerAddress(domain: string): Promise<{ code: number; stdout: string; stderr: string }> {
    return execute(['address', 'register', domain]);
  }
  
  /**
   * Resolve a dark address
   */
  static async resolveAddress(domain: string): Promise<{ code: number; stdout: string; stderr: string }> {
    return execute(['address', 'resolve', domain]);
  }
  
  /**
   * Generate a shadow address
   */
  static async generateShadowAddress(ttl?: number): Promise<{ code: number; stdout: string; stderr: string }> {
    const args = ['address', 'shadow'];
    if (ttl) {
      args.push('--ttl', ttl.toString());
    }
    return execute(args);
  }
  
  /**
   * Create a quantum fingerprint
   */
  static async createFingerprint(data: string): Promise<{ code: number; stdout: string; stderr: string }> {
    return execute(['address', 'fingerprint', '--data', data]);
  }
  
  /**
   * Execute a raw command
   */
  static async raw(args: string[]): Promise<{ code: number; stdout: string; stderr: string }> {
    return execute(args);
  }
}

/**
 * Check if QuDAG binary is installed
 */
export function isInstalled(): boolean {
  const binaryPath = getBinaryPath();
  try {
    require('fs').accessSync(binaryPath, require('fs').constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

// Export types
export interface QuDAGResult {
  code: number;
  stdout: string;
  stderr: string;
}

export interface PlatformInfo {
  platform: string;
  arch: string;
  targetTriple: string;
  binaryName: string;
  binaryPath: string;
}