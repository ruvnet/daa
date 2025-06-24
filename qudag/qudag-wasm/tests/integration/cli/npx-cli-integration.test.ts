import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from 'vitest';
import { exec, spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdtemp, rm, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const execAsync = promisify(exec);

describe('NPX CLI Integration', () => {
  let testDir: string;
  let cliPath: string;
  
  beforeAll(async () => {
    // Build the CLI if not already built
    cliPath = join(process.cwd(), 'dist', 'cli.js');
    
    try {
      await execAsync('npm run build', { cwd: process.cwd() });
    } catch (error) {
      console.warn('Build failed, tests may use existing build');
    }
  });
  
  beforeEach(async () => {
    // Create temporary directory for each test
    testDir = await mkdtemp(join(tmpdir(), 'qudag-test-'));
  });
  
  afterEach(async () => {
    // Clean up temporary directory
    await rm(testDir, { recursive: true, force: true });
  });
  
  describe('CLI Initialization', () => {
    it('should run via npx', async () => {
      const { stdout, stderr } = await execAsync('npx qudag --version', {
        cwd: testDir
      });
      
      expect(stderr).toBe('');
      expect(stdout).toMatch(/QuDAG v\d+\.\d+\.\d+/);
    });
    
    it('should show help information', async () => {
      const { stdout } = await execAsync('npx qudag --help', {
        cwd: testDir
      });
      
      expect(stdout).toContain('QuDAG - Quantum-Resistant Distributed Acyclic Graph');
      expect(stdout).toContain('Commands:');
      expect(stdout).toContain('init');
      expect(stdout).toContain('create');
      expect(stdout).toContain('add');
      expect(stdout).toContain('query');
      expect(stdout).toContain('serve');
    });
    
    it('should handle unknown commands gracefully', async () => {
      try {
        await execAsync('npx qudag unknown-command', { cwd: testDir });
        expect.fail('Should have thrown error');
      } catch (error) {
        expect(error.stderr).toContain('Unknown command');
        expect(error.code).toBe(1);
      }
    });
  });
  
  describe('Vault Operations', () => {
    it('should initialize a new vault', async () => {
      const { stdout } = await execAsync('npx qudag init --name test-vault', {
        cwd: testDir
      });
      
      expect(stdout).toContain('Vault initialized successfully');
      expect(stdout).toContain('test-vault');
      
      // Verify vault file was created
      const vaultFile = join(testDir, '.qudag', 'vault.json');
      const vaultData = JSON.parse(await readFile(vaultFile, 'utf-8'));
      
      expect(vaultData.name).toBe('test-vault');
      expect(vaultData.version).toBeDefined();
      expect(vaultData.created).toBeDefined();
    });
    
    it('should create vault with custom configuration', async () => {
      const config = {
        crypto: {
          algorithm: 'ML-KEM-768',
          signatureAlgorithm: 'ML-DSA-65'
        },
        consensus: {
          algorithm: 'avalanche',
          k: 10,
          alpha: 8
        }
      };
      
      await writeFile(
        join(testDir, 'qudag.config.json'),
        JSON.stringify(config, null, 2)
      );
      
      const { stdout } = await execAsync('npx qudag init --config qudag.config.json', {
        cwd: testDir
      });
      
      expect(stdout).toContain('Using custom configuration');
      
      const vaultFile = join(testDir, '.qudag', 'vault.json');
      const vaultData = JSON.parse(await readFile(vaultFile, 'utf-8'));
      
      expect(vaultData.config.crypto.algorithm).toBe('ML-KEM-768');
      expect(vaultData.config.consensus.algorithm).toBe('avalanche');
    });
  });
  
  describe('DAG Operations', () => {
    beforeEach(async () => {
      // Initialize vault for DAG operations
      await execAsync('npx qudag init', { cwd: testDir });
    });
    
    it('should add vertex to DAG', async () => {
      const { stdout } = await execAsync(
        'npx qudag add --data "Hello, QuDAG!"',
        { cwd: testDir }
      );
      
      expect(stdout).toContain('Vertex added successfully');
      expect(stdout).toMatch(/ID: [0-9a-f]{64}/);
    });
    
    it('should add vertex with file data', async () => {
      const testFile = join(testDir, 'test.txt');
      await writeFile(testFile, 'File content for QuDAG');
      
      const { stdout } = await execAsync(
        `npx qudag add --file ${testFile}`,
        { cwd: testDir }
      );
      
      expect(stdout).toContain('Vertex added successfully');
      const idMatch = stdout.match(/ID: ([0-9a-f]{64})/);
      expect(idMatch).toBeTruthy();
      
      // Verify vertex content
      const vertexId = idMatch[1];
      const { stdout: queryOut } = await execAsync(
        `npx qudag query ${vertexId}`,
        { cwd: testDir }
      );
      
      expect(queryOut).toContain('File content for QuDAG');
    });
    
    it('should add vertex with parents', async () => {
      // Add first vertex
      const { stdout: out1 } = await execAsync(
        'npx qudag add --data "Parent vertex"',
        { cwd: testDir }
      );
      const parentId = out1.match(/ID: ([0-9a-f]{64})/)[1];
      
      // Add child vertex
      const { stdout: out2 } = await execAsync(
        `npx qudag add --data "Child vertex" --parents ${parentId}`,
        { cwd: testDir }
      );
      
      expect(out2).toContain('Vertex added successfully');
      const childId = out2.match(/ID: ([0-9a-f]{64})/)[1];
      
      // Query child to verify parent
      const { stdout: queryOut } = await execAsync(
        `npx qudag query ${childId} --format json`,
        { cwd: testDir }
      );
      
      const vertex = JSON.parse(queryOut);
      expect(vertex.parents).toContain(parentId);
    });
    
    it('should list DAG tips', async () => {
      // Add multiple vertices
      for (let i = 0; i < 3; i++) {
        await execAsync(
          `npx qudag add --data "Vertex ${i}"`,
          { cwd: testDir }
        );
      }
      
      const { stdout } = await execAsync('npx qudag tips', { cwd: testDir });
      
      expect(stdout).toContain('Current tips:');
      const tips = stdout.match(/[0-9a-f]{64}/g);
      expect(tips).toHaveLength(3);
    });
  });
  
  describe('Query Operations', () => {
    let vertexIds: string[] = [];
    
    beforeEach(async () => {
      await execAsync('npx qudag init', { cwd: testDir });
      
      // Create test vertices
      for (let i = 0; i < 5; i++) {
        const { stdout } = await execAsync(
          `npx qudag add --data "Test vertex ${i}" --metadata '{"index": ${i}}'`,
          { cwd: testDir }
        );
        const id = stdout.match(/ID: ([0-9a-f]{64})/)[1];
        vertexIds.push(id);
      }
    });
    
    it('should query vertex by ID', async () => {
      const { stdout } = await execAsync(
        `npx qudag query ${vertexIds[0]}`,
        { cwd: testDir }
      );
      
      expect(stdout).toContain('Test vertex 0');
      expect(stdout).toContain('Height: 0');
      expect(stdout).toContain('Parents: none');
    });
    
    it('should export DAG structure', async () => {
      const exportFile = join(testDir, 'dag-export.json');
      
      await execAsync(
        `npx qudag export --output ${exportFile}`,
        { cwd: testDir }
      );
      
      const exportData = JSON.parse(await readFile(exportFile, 'utf-8'));
      
      expect(exportData.vertices).toHaveLength(5);
      expect(exportData.metadata.exported).toBeDefined();
      expect(exportData.metadata.version).toBeDefined();
    });
    
    it('should import DAG from file', async () => {
      // Export first
      const exportFile = join(testDir, 'dag-export.json');
      await execAsync(`npx qudag export --output ${exportFile}`, { cwd: testDir });
      
      // Create new vault
      const importDir = join(testDir, 'import-test');
      await execAsync(`mkdir -p ${importDir}`);
      await execAsync('npx qudag init', { cwd: importDir });
      
      // Import
      const { stdout } = await execAsync(
        `npx qudag import --input ${exportFile}`,
        { cwd: importDir }
      );
      
      expect(stdout).toContain('Import successful');
      expect(stdout).toContain('5 vertices imported');
    });
  });
  
  describe('Server Mode', () => {
    it('should start HTTP server', async () => {
      const serverProcess = spawn('npx', ['qudag', 'serve', '--port', '0'], {
        cwd: testDir
      });
      
      let serverOutput = '';
      serverProcess.stdout.on('data', (data) => {
        serverOutput += data.toString();
      });
      
      // Wait for server to start
      await new Promise<void>((resolve) => {
        const checkServer = setInterval(() => {
          if (serverOutput.includes('Server listening on')) {
            clearInterval(checkServer);
            resolve();
          }
        }, 100);
      });
      
      // Extract port from output
      const portMatch = serverOutput.match(/Server listening on port (\d+)/);
      expect(portMatch).toBeTruthy();
      const port = portMatch[1];
      
      // Test server endpoint
      const response = await fetch(`http://localhost:${port}/api/status`);
      expect(response.ok).toBe(true);
      
      const status = await response.json();
      expect(status.version).toBeDefined();
      expect(status.vertices).toBe(0);
      
      // Clean up
      serverProcess.kill();
      await new Promise(resolve => serverProcess.on('close', resolve));
    });
  });
  
  describe('Interactive Mode', () => {
    it('should support interactive REPL', async () => {
      const replProcess = spawn('npx', ['qudag', 'repl'], {
        cwd: testDir
      });
      
      let output = '';
      replProcess.stdout.on('data', (data) => {
        output += data.toString();
      });
      
      // Wait for REPL prompt
      await testUtils.waitForCondition(() => output.includes('qudag>'));
      
      // Send command
      replProcess.stdin.write('status\n');
      
      await testUtils.waitForCondition(() => output.includes('Vault status:'));
      expect(output).toContain('Vertices: 0');
      
      // Exit REPL
      replProcess.stdin.write('exit\n');
      await new Promise(resolve => replProcess.on('close', resolve));
    });
  });
  
  describe('Performance Features', () => {
    it('should handle batch operations', async () => {
      await execAsync('npx qudag init', { cwd: testDir });
      
      // Create batch file
      const batchFile = join(testDir, 'batch.jsonl');
      const batchData = Array(100).fill(0).map((_, i) => 
        JSON.stringify({
          operation: 'add',
          data: `Batch vertex ${i}`,
          metadata: { batch: true, index: i }
        })
      ).join('\n');
      
      await writeFile(batchFile, batchData);
      
      const start = Date.now();
      const { stdout } = await execAsync(
        `npx qudag batch --input ${batchFile}`,
        { cwd: testDir }
      );
      const duration = Date.now() - start;
      
      expect(stdout).toContain('Batch processing complete');
      expect(stdout).toContain('100 operations processed');
      expect(duration).toBeLessThan(5000); // Should complete within 5 seconds
    });
    
    it('should support parallel operations', async () => {
      await execAsync('npx qudag init --workers 4', { cwd: testDir });
      
      // Run parallel adds
      const promises = Array(20).fill(0).map((_, i) => 
        execAsync(`npx qudag add --data "Parallel ${i}"`, { cwd: testDir })
      );
      
      const results = await Promise.all(promises);
      
      // All should succeed
      results.forEach(result => {
        expect(result.stdout).toContain('Vertex added successfully');
      });
      
      // Verify count
      const { stdout } = await execAsync('npx qudag status', { cwd: testDir });
      expect(stdout).toContain('Vertices: 20');
    });
  });
  
  describe('Error Scenarios', () => {
    it('should handle missing vault gracefully', async () => {
      try {
        await execAsync('npx qudag add --data "test"', { cwd: testDir });
        expect.fail('Should have thrown error');
      } catch (error) {
        expect(error.stderr).toContain('No vault found');
        expect(error.stderr).toContain('Run "qudag init" first');
      }
    });
    
    it('should validate vertex data', async () => {
      await execAsync('npx qudag init', { cwd: testDir });
      
      try {
        await execAsync('npx qudag add', { cwd: testDir });
        expect.fail('Should have thrown error');
      } catch (error) {
        expect(error.stderr).toContain('No data provided');
      }
    });
    
    it('should handle invalid parent references', async () => {
      await execAsync('npx qudag init', { cwd: testDir });
      
      try {
        await execAsync(
          'npx qudag add --data "test" --parents invalidid',
          { cwd: testDir }
        );
        expect.fail('Should have thrown error');
      } catch (error) {
        expect(error.stderr).toContain('Invalid parent ID');
      }
    });
  });
});