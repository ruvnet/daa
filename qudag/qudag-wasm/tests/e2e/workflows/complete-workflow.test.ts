import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { chromium, Browser, Page } from 'playwright';
import { createServer } from 'node:http';
import { join } from 'node:path';
import type { QuDAG, VertexId } from '@/types';

describe('Complete E2E Workflow', () => {
  let browser: Browser;
  let page: Page;
  let serverUrl: string;
  let dag: QuDAG;
  
  beforeAll(async () => {
    // Start test server
    const server = createServer((req, res) => {
      // Serve test page
      if (req.url === '/') {
        res.writeHead(200, { 'Content-Type': 'text/html' });
        res.end(`
          <!DOCTYPE html>
          <html>
            <head>
              <title>QuDAG E2E Test</title>
              <script type="module">
                import { QuDAG } from '/dist/qudag.js';
                window.QuDAG = QuDAG;
              </script>
            </head>
            <body>
              <h1>QuDAG E2E Test Page</h1>
              <div id="status">Loading...</div>
              <div id="results"></div>
            </body>
          </html>
        `);
      } else if (req.url.startsWith('/dist/')) {
        // Serve built files
        const fs = await import('node:fs/promises');
        const filePath = join(process.cwd(), req.url);
        try {
          const content = await fs.readFile(filePath);
          res.writeHead(200);
          res.end(content);
        } catch {
          res.writeHead(404);
          res.end('Not found');
        }
      }
    });
    
    await new Promise<void>(resolve => {
      server.listen(0, () => {
        const address = server.address();
        serverUrl = `http://localhost:${address.port}`;
        resolve();
      });
    });
    
    // Launch browser
    browser = await chromium.launch({
      headless: true
    });
    
    page = await browser.newPage();
    
    // Enable console logging
    page.on('console', msg => console.log('Browser console:', msg.text()));
  });
  
  afterAll(async () => {
    await browser.close();
  });
  
  describe('Browser-based Workflow', () => {
    it('should complete full vault workflow in browser', async () => {
      await page.goto(serverUrl);
      
      // Wait for QuDAG to load
      await page.waitForFunction(() => window.QuDAG !== undefined);
      
      // Execute workflow
      const result = await page.evaluate(async () => {
        const status = document.getElementById('status');
        const results = document.getElementById('results');
        
        try {
          // Step 1: Initialize QuDAG
          status.textContent = 'Initializing QuDAG...';
          const dag = await window.QuDAG.create({
            persistence: 'indexeddb',
            name: 'e2e-test-vault'
          });
          
          // Step 2: Create initial vertices
          status.textContent = 'Creating vertices...';
          const vertices = [];
          
          const genesis = await dag.addVertex({
            payload: 'Genesis block',
            metadata: { timestamp: Date.now() }
          });
          vertices.push(genesis);
          
          // Create a chain
          for (let i = 1; i <= 10; i++) {
            const vertex = await dag.addVertex({
              payload: `Block ${i}`,
              parents: [vertices[vertices.length - 1]],
              metadata: { 
                index: i,
                timestamp: Date.now()
              }
            });
            vertices.push(vertex);
          }
          
          // Step 3: Create branches
          status.textContent = 'Creating branches...';
          const branch1 = await dag.addVertex({
            payload: 'Branch 1',
            parents: [vertices[5]],
            metadata: { branch: 'alpha' }
          });
          
          const branch2 = await dag.addVertex({
            payload: 'Branch 2',
            parents: [vertices[5]],
            metadata: { branch: 'beta' }
          });
          
          // Step 4: Merge branches
          const merge = await dag.addVertex({
            payload: 'Merge point',
            parents: [branch1, branch2],
            metadata: { type: 'merge' }
          });
          
          // Step 5: Test consensus
          status.textContent = 'Testing consensus...';
          const confidence = await dag.consensus.getConfidence(merge);
          
          // Simulate voting
          for (let i = 0; i < 10; i++) {
            await dag.consensus.simulateVote(merge, true, `peer-${i}`);
          }
          
          const updatedConfidence = await dag.consensus.getConfidence(merge);
          
          // Step 6: Query operations
          status.textContent = 'Running queries...';
          const tips = await dag.getTips();
          const ancestors = await dag.getAncestors(merge);
          const path = await dag.findPaths(genesis, merge);
          
          // Step 7: Export/Import test
          status.textContent = 'Testing export/import...';
          const exported = await dag.exportJSON();
          
          // Create new DAG and import
          const dag2 = await window.QuDAG.create({
            persistence: 'memory',
            name: 'import-test'
          });
          
          await dag2.importJSON(exported);
          const importedVertex = await dag2.getVertex(merge);
          
          // Step 8: Performance test
          status.textContent = 'Performance testing...';
          const perfStart = performance.now();
          
          const batchVertices = [];
          for (let i = 0; i < 100; i++) {
            batchVertices.push({
              payload: `Perf test ${i}`,
              parents: i > 0 ? [vertices[Math.floor(Math.random() * vertices.length)]] : []
            });
          }
          
          await dag.addVertices(batchVertices);
          const perfDuration = performance.now() - perfStart;
          
          // Return results
          status.textContent = 'Complete!';
          return {
            success: true,
            vertexCount: await dag.getVertexCount(),
            tips: tips.length,
            ancestorCount: ancestors.size,
            pathLength: path[0]?.length || 0,
            confidenceBefore: confidence.value,
            confidenceAfter: updatedConfidence.value,
            importSuccess: importedVertex !== null,
            perfDuration,
            perfOpsPerSecond: (100 / perfDuration) * 1000
          };
        } catch (error) {
          status.textContent = 'Error: ' + error.message;
          return {
            success: false,
            error: error.message
          };
        }
      });
      
      // Verify results
      expect(result.success).toBe(true);
      expect(result.vertexCount).toBeGreaterThan(100);
      expect(result.tips).toBeGreaterThan(0);
      expect(result.ancestorCount).toBeGreaterThan(5);
      expect(result.pathLength).toBeGreaterThan(0);
      expect(result.confidenceAfter).toBeGreaterThan(result.confidenceBefore);
      expect(result.importSuccess).toBe(true);
      expect(result.perfOpsPerSecond).toBeGreaterThan(100); // At least 100 ops/sec
    });
    
    it('should persist data across sessions', async () => {
      // First session - create data
      await page.goto(serverUrl);
      await page.waitForFunction(() => window.QuDAG !== undefined);
      
      const firstSessionData = await page.evaluate(async () => {
        const dag = await window.QuDAG.create({
          persistence: 'indexeddb',
          name: 'persistence-test'
        });
        
        const vertices = [];
        for (let i = 0; i < 5; i++) {
          vertices.push(await dag.addVertex({
            payload: `Persistent vertex ${i}`
          }));
        }
        
        return {
          vertices,
          count: await dag.getVertexCount()
        };
      });
      
      expect(firstSessionData.count).toBe(5);
      
      // Reload page to simulate new session
      await page.reload();
      await page.waitForFunction(() => window.QuDAG !== undefined);
      
      // Second session - verify data persisted
      const secondSessionData = await page.evaluate(async () => {
        const dag = await window.QuDAG.create({
          persistence: 'indexeddb',
          name: 'persistence-test'
        });
        
        const count = await dag.getVertexCount();
        const vertices = await dag.getTips();
        
        return { count, vertices };
      });
      
      expect(secondSessionData.count).toBe(5);
      expect(secondSessionData.vertices).toHaveLength(5);
    });
  });
  
  describe('Node.js Workflow', () => {
    beforeAll(async () => {
      const { QuDAG } = await import('@/index');
      dag = await QuDAG.create({
        persistence: 'filesystem',
        path: './test-vault'
      });
    });
    
    afterAll(async () => {
      dag.dispose();
      // Clean up test vault
      const fs = await import('node:fs/promises');
      await fs.rm('./test-vault', { recursive: true, force: true });
    });
    
    it('should complete distributed vault workflow', async () => {
      // Step 1: Create user identities
      const users = [];
      for (let i = 0; i < 3; i++) {
        const keyPair = await dag.crypto.generateKeyPair('ML-KEM-768');
        users.push({
          id: `user-${i}`,
          keyPair
        });
      }
      
      // Step 2: Store encrypted secrets
      const secrets = [];
      for (const user of users) {
        const secret = {
          type: 'password',
          value: `secret-${user.id}`,
          owner: user.id
        };
        
        const encrypted = await dag.crypto.encrypt(
          user.keyPair.publicKey,
          new TextEncoder().encode(JSON.stringify(secret))
        );
        
        const vertex = await dag.addVertex({
          payload: encrypted,
          metadata: {
            type: 'encrypted-secret',
            owner: user.id,
            algorithm: 'ML-KEM-768'
          }
        });
        
        secrets.push({ vertex, user });
      }
      
      // Step 3: Share secrets between users
      const sharedSecret = await dag.addVertex({
        payload: 'Shared configuration',
        parents: secrets.map(s => s.vertex),
        metadata: {
          type: 'shared-secret',
          participants: users.map(u => u.id)
        }
      });
      
      // Step 4: Create access control entries
      for (const user of users) {
        const signature = await dag.crypto.sign(
          user.keyPair.secretKey,
          new TextEncoder().encode(sharedSecret)
        );
        
        await dag.addVertex({
          payload: signature,
          parents: [sharedSecret],
          metadata: {
            type: 'access-grant',
            grantee: user.id,
            permissions: ['read', 'update']
          }
        });
      }
      
      // Step 5: Verify consensus on shared secret
      for (let round = 0; round < 5; round++) {
        for (const user of users) {
          await dag.consensus.simulateVote(sharedSecret, true, user.id);
        }
        await dag.consensus.advanceRound();
      }
      
      const consensusStatus = await dag.consensus.getConsensusStatus(sharedSecret);
      expect(consensusStatus).toBe('accepted');
      
      // Step 6: Test conflict resolution
      const conflict1 = await dag.addVertex({
        payload: 'Config version 1',
        parents: [sharedSecret],
        metadata: { 
          type: 'config-update',
          version: 1,
          conflictSet: 'config'
        }
      });
      
      const conflict2 = await dag.addVertex({
        payload: 'Config version 2',
        parents: [sharedSecret],
        metadata: { 
          type: 'config-update',
          version: 2,
          conflictSet: 'config'
        }
      });
      
      // Users vote on preferred version
      await dag.consensus.simulateVote(conflict1, true, users[0].id);
      await dag.consensus.simulateVote(conflict1, true, users[1].id);
      await dag.consensus.simulateVote(conflict2, true, users[2].id);
      
      const winner = await dag.consensus.resolveConflict([conflict1, conflict2]);
      expect(winner).toBe(conflict1);
      
      // Step 7: Create audit trail
      const auditEntries = [];
      for (const event of ['create', 'share', 'update', 'access']) {
        const entry = await dag.addVertex({
          payload: {
            event,
            timestamp: Date.now(),
            actor: users[0].id,
            target: sharedSecret
          },
          parents: auditEntries.length > 0 ? [auditEntries[auditEntries.length - 1]] : [sharedSecret],
          metadata: {
            type: 'audit-log',
            immutable: true
          }
        });
        auditEntries.push(entry);
      }
      
      // Verify complete workflow
      const stats = {
        totalVertices: await dag.getVertexCount(),
        depth: await dag.getDepth(),
        tips: await dag.getTips(),
        consensusStats: await dag.consensus.getStats()
      };
      
      expect(stats.totalVertices).toBeGreaterThan(10);
      expect(stats.depth).toBeGreaterThan(2);
      expect(stats.consensusStats.consensusReached).toBeGreaterThan(0);
    });
  });
  
  describe('Stress Testing', () => {
    it('should handle high-throughput operations', async () => {
      const { QuDAG } = await import('@/index');
      const stressDAG = await QuDAG.create({
        persistence: 'memory',
        threading: { enabled: true, workers: 4 }
      });
      
      const startTime = Date.now();
      const operations = [];
      
      // Concurrent write operations
      for (let i = 0; i < 1000; i++) {
        operations.push(stressDAG.addVertex({
          payload: `Stress test ${i}`,
          metadata: { timestamp: Date.now() }
        }));
        
        // Add some reads
        if (i % 10 === 0 && operations.length > 10) {
          const randomVertex = await operations[Math.floor(Math.random() * 10)];
          operations.push(stressDAG.getVertex(randomVertex));
        }
      }
      
      const results = await Promise.all(operations);
      const duration = Date.now() - startTime;
      
      expect(results.filter(r => r !== null)).toHaveLength(operations.length);
      expect(duration).toBeLessThan(10000); // Complete within 10 seconds
      
      const throughput = (operations.length / duration) * 1000;
      expect(throughput).toBeGreaterThan(100); // At least 100 ops/sec
      
      stressDAG.dispose();
    });
    
    it('should maintain consistency under concurrent access', async () => {
      const { QuDAG } = await import('@/index');
      const concurrentDAG = await QuDAG.create({
        persistence: 'memory'
      });
      
      // Create base structure
      const root = await concurrentDAG.addVertex({ payload: 'Root' });
      
      // Concurrent branch creation
      const branches = await Promise.all(
        Array(10).fill(0).map(async (_, i) => {
          const branch = [];
          let parent = root;
          
          for (let j = 0; j < 10; j++) {
            const vertex = await concurrentDAG.addVertex({
              payload: `Branch ${i} - Node ${j}`,
              parents: [parent]
            });
            branch.push(vertex);
            parent = vertex;
          }
          
          return branch;
        })
      );
      
      // Verify DAG integrity
      const tips = await concurrentDAG.getTips();
      expect(tips).toHaveLength(10); // One tip per branch
      
      // Verify no cycles
      for (const tip of tips) {
        const ancestors = await concurrentDAG.getAncestors(tip);
        expect(ancestors.has(tip)).toBe(false); // No self-reference
      }
      
      // Verify all paths lead to root
      for (const branch of branches) {
        for (const vertex of branch) {
          const paths = await concurrentDAG.findPaths(root, vertex);
          expect(paths.length).toBeGreaterThan(0);
        }
      }
      
      concurrentDAG.dispose();
    });
  });
});