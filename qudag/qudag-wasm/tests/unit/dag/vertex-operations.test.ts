import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import type { 
  QuDAG, 
  Vertex, 
  VertexId, 
  VertexInput,
  ValidationError 
} from '@/types';

describe('DAG Vertex Operations', () => {
  let dag: QuDAG;
  
  beforeEach(async () => {
    const { QuDAG } = await import('@/dag');
    dag = await QuDAG.create();
  });
  
  afterEach(() => {
    dag.dispose();
  });
  
  describe('Vertex Creation', () => {
    it('should create a genesis vertex', async () => {
      const vertex = await dag.addVertex({
        payload: 'Genesis block',
        parents: []
      });
      
      expect(vertex).toBeValidVertexId();
      
      const retrieved = await dag.getVertex(vertex);
      expect(retrieved).toBeDefined();
      expect(retrieved.payload).toBe('Genesis block');
      expect(retrieved.parents).toHaveLength(0);
      expect(retrieved.height).toBe(0);
    });
    
    it('should create vertex with single parent', async () => {
      const parent = await dag.addVertex({
        payload: 'Parent',
        parents: []
      });
      
      const child = await dag.addVertex({
        payload: 'Child',
        parents: [parent]
      });
      
      const retrieved = await dag.getVertex(child);
      expect(retrieved.parents).toHaveLength(1);
      expect(retrieved.parents[0]).toBe(parent);
      expect(retrieved.height).toBe(1);
    });
    
    it('should create vertex with multiple parents', async () => {
      const parent1 = await dag.addVertex({ payload: 'Parent 1', parents: [] });
      const parent2 = await dag.addVertex({ payload: 'Parent 2', parents: [] });
      const parent3 = await dag.addVertex({ payload: 'Parent 3', parents: [] });
      
      const child = await dag.addVertex({
        payload: 'Multi-parent child',
        parents: [parent1, parent2, parent3]
      });
      
      const retrieved = await dag.getVertex(child);
      expect(retrieved.parents).toHaveLength(3);
      expect(retrieved.parents).toContain(parent1);
      expect(retrieved.parents).toContain(parent2);
      expect(retrieved.parents).toContain(parent3);
    });
    
    it('should handle binary payload', async () => {
      const binaryData = new Uint8Array([1, 2, 3, 4, 5]);
      
      const vertex = await dag.addVertex({
        payload: binaryData,
        parents: []
      });
      
      const retrieved = await dag.getVertex(vertex);
      expect(retrieved.payload).toBeInstanceOf(Uint8Array);
      expect(retrieved.payload).toEqual(binaryData);
    });
    
    it('should add metadata to vertices', async () => {
      const vertex = await dag.addVertex({
        payload: 'Test',
        parents: [],
        metadata: {
          author: 'test-user',
          timestamp: Date.now(),
          tags: ['important', 'test']
        }
      });
      
      const retrieved = await dag.getVertex(vertex);
      expect(retrieved.metadata).toBeDefined();
      expect(retrieved.metadata.author).toBe('test-user');
      expect(retrieved.metadata.tags).toContain('important');
    });
  });
  
  describe('Vertex Validation', () => {
    it('should reject vertex with non-existent parent', async () => {
      const fakeParent = 'a'.repeat(64); // Valid format but doesn't exist
      
      await expect(dag.addVertex({
        payload: 'Orphan',
        parents: [fakeParent]
      })).rejects.toThrow('Parent vertex not found');
    });
    
    it('should reject vertex creating cycle', async () => {
      const v1 = await dag.addVertex({ payload: 'V1', parents: [] });
      const v2 = await dag.addVertex({ payload: 'V2', parents: [v1] });
      
      // Try to make v1 depend on v2 (creating a cycle)
      await expect(dag.updateVertex(v1, {
        parents: [v2]
      })).rejects.toThrow('Cycle detected');
    });
    
    it('should reject duplicate parents', async () => {
      const parent = await dag.addVertex({ payload: 'Parent', parents: [] });
      
      await expect(dag.addVertex({
        payload: 'Child',
        parents: [parent, parent]
      })).rejects.toThrow('Duplicate parents not allowed');
    });
    
    it('should enforce maximum parent limit', async () => {
      const parents = [];
      for (let i = 0; i < 20; i++) {
        parents.push(await dag.addVertex({ 
          payload: `Parent ${i}`, 
          parents: [] 
        }));
      }
      
      await expect(dag.addVertex({
        payload: 'Too many parents',
        parents
      })).rejects.toThrow('Exceeds maximum parent limit');
    });
    
    it('should validate payload size', async () => {
      const largePayload = new Uint8Array(10 * 1024 * 1024); // 10MB
      
      await expect(dag.addVertex({
        payload: largePayload,
        parents: []
      })).rejects.toThrow('Payload size exceeds limit');
    });
  });
  
  describe('Vertex Retrieval', () => {
    it('should retrieve vertex by ID', async () => {
      const id = await dag.addVertex({
        payload: 'Test vertex',
        parents: []
      });
      
      const vertex = await dag.getVertex(id);
      expect(vertex).toBeDefined();
      expect(vertex.id).toBe(id);
      expect(vertex.payload).toBe('Test vertex');
    });
    
    it('should return null for non-existent vertex', async () => {
      const fakeId = 'f'.repeat(64);
      const vertex = await dag.getVertex(fakeId);
      expect(vertex).toBeNull();
    });
    
    it('should check vertex existence', async () => {
      const id = await dag.addVertex({
        payload: 'Exists',
        parents: []
      });
      
      expect(await dag.hasVertex(id)).toBe(true);
      expect(await dag.hasVertex('f'.repeat(64))).toBe(false);
    });
    
    it('should retrieve multiple vertices in batch', async () => {
      const ids = [];
      for (let i = 0; i < 10; i++) {
        ids.push(await dag.addVertex({
          payload: `Vertex ${i}`,
          parents: []
        }));
      }
      
      const vertices = await dag.getVertices(ids);
      expect(vertices).toHaveLength(10);
      vertices.forEach((vertex, i) => {
        expect(vertex.payload).toBe(`Vertex ${i}`);
      });
    });
    
    it('should handle mixed batch with non-existent vertices', async () => {
      const validId = await dag.addVertex({
        payload: 'Valid',
        parents: []
      });
      
      const fakeId = 'f'.repeat(64);
      
      const vertices = await dag.getVertices([validId, fakeId]);
      expect(vertices).toHaveLength(2);
      expect(vertices[0]).toBeDefined();
      expect(vertices[0].payload).toBe('Valid');
      expect(vertices[1]).toBeNull();
    });
  });
  
  describe('DAG Traversal', () => {
    let root: VertexId;
    let level1: VertexId[];
    let level2: VertexId[];
    
    beforeEach(async () => {
      // Create test DAG structure
      //       root
      //      /  |  \
      //    l1a l1b l1c
      //    / \   |   |
      //  l2a l2b l2c l2d
      
      root = await dag.addVertex({ payload: 'root', parents: [] });
      
      level1 = await Promise.all([
        dag.addVertex({ payload: 'l1a', parents: [root] }),
        dag.addVertex({ payload: 'l1b', parents: [root] }),
        dag.addVertex({ payload: 'l1c', parents: [root] })
      ]);
      
      level2 = await Promise.all([
        dag.addVertex({ payload: 'l2a', parents: [level1[0]] }),
        dag.addVertex({ payload: 'l2b', parents: [level1[0]] }),
        dag.addVertex({ payload: 'l2c', parents: [level1[1]] }),
        dag.addVertex({ payload: 'l2d', parents: [level1[2]] })
      ]);
    });
    
    it('should get direct children', async () => {
      const children = await dag.getChildren(root);
      expect(children).toHaveLength(3);
      expect(children).toContain(level1[0]);
      expect(children).toContain(level1[1]);
      expect(children).toContain(level1[2]);
    });
    
    it('should get all descendants', async () => {
      const descendants = await dag.getDescendants(root);
      expect(descendants.size).toBe(7); // 3 level1 + 4 level2
      
      level1.forEach(id => expect(descendants.has(id)).toBe(true));
      level2.forEach(id => expect(descendants.has(id)).toBe(true));
    });
    
    it('should get descendants with depth limit', async () => {
      const descendants = await dag.getDescendants(root, 1);
      expect(descendants.size).toBe(3); // Only level1
      
      level1.forEach(id => expect(descendants.has(id)).toBe(true));
      level2.forEach(id => expect(descendants.has(id)).toBe(false));
    });
    
    it('should get ancestors', async () => {
      const ancestors = await dag.getAncestors(level2[0]);
      expect(ancestors.size).toBe(2); // l1a and root
      expect(ancestors.has(level1[0])).toBe(true);
      expect(ancestors.has(root)).toBe(true);
    });
    
    it('should find paths between vertices', async () => {
      const paths = await dag.findPaths(root, level2[0]);
      expect(paths).toHaveLength(1);
      expect(paths[0]).toEqual([root, level1[0], level2[0]]);
    });
    
    it('should detect if vertex is ancestor', async () => {
      expect(await dag.isAncestor(root, level2[0])).toBe(true);
      expect(await dag.isAncestor(level2[0], root)).toBe(false);
      expect(await dag.isAncestor(level1[0], level1[1])).toBe(false);
    });
  });
  
  describe('Tips Management', () => {
    it('should identify tips correctly', async () => {
      const v1 = await dag.addVertex({ payload: 'V1', parents: [] });
      const v2 = await dag.addVertex({ payload: 'V2', parents: [] });
      const v3 = await dag.addVertex({ payload: 'V3', parents: [v1, v2] });
      
      const tips = await dag.getTips();
      expect(tips).toHaveLength(1);
      expect(tips[0]).toBe(v3);
    });
    
    it('should update tips when new vertices are added', async () => {
      const v1 = await dag.addVertex({ payload: 'V1', parents: [] });
      
      let tips = await dag.getTips();
      expect(tips).toEqual([v1]);
      
      const v2 = await dag.addVertex({ payload: 'V2', parents: [v1] });
      
      tips = await dag.getTips();
      expect(tips).toEqual([v2]);
      
      const v3 = await dag.addVertex({ payload: 'V3', parents: [] });
      
      tips = await dag.getTips();
      expect(tips).toHaveLength(2);
      expect(tips).toContain(v2);
      expect(tips).toContain(v3);
    });
    
    it('should handle tips with different heights', async () => {
      const v1 = await dag.addVertex({ payload: 'V1', parents: [] });
      const v2 = await dag.addVertex({ payload: 'V2', parents: [v1] });
      const v3 = await dag.addVertex({ payload: 'V3', parents: [v2] });
      const v4 = await dag.addVertex({ payload: 'V4', parents: [] });
      
      const tips = await dag.getTips();
      expect(tips).toHaveLength(2);
      expect(tips).toContain(v3);
      expect(tips).toContain(v4);
      
      const tipVertices = await Promise.all(tips.map(id => dag.getVertex(id)));
      expect(tipVertices[0].height).not.toBe(tipVertices[1].height);
    });
  });
  
  describe('Batch Operations', () => {
    it('should add multiple vertices in batch', async () => {
      const inputs: VertexInput[] = Array(100).fill(0).map((_, i) => ({
        payload: `Vertex ${i}`,
        parents: []
      }));
      
      const ids = await dag.addVertices(inputs);
      expect(ids).toHaveLength(100);
      
      // Verify all were created
      const vertices = await dag.getVertices(ids);
      vertices.forEach((vertex, i) => {
        expect(vertex.payload).toBe(`Vertex ${i}`);
      });
    });
    
    it('should maintain consistency in batch operations', async () => {
      const genesis = await dag.addVertex({ payload: 'Genesis', parents: [] });
      
      // Create vertices that depend on each other within the batch
      const inputs: VertexInput[] = [
        { payload: 'B1', parents: [genesis] },
        { payload: 'B2', parents: [genesis] },
        { payload: 'B3', parents: [] } // Reference to B1 will be added after
      ];
      
      const ids = await dag.addVertices(inputs);
      
      // Update B3 to depend on B1
      await dag.updateVertex(ids[2], {
        parents: [ids[0]]
      });
      
      const b3 = await dag.getVertex(ids[2]);
      expect(b3.parents).toContain(ids[0]);
    });
    
    it('should rollback batch on failure', async () => {
      const validInputs: VertexInput[] = [
        { payload: 'Valid1', parents: [] },
        { payload: 'Valid2', parents: [] }
      ];
      
      const invalidInputs: VertexInput[] = [
        ...validInputs,
        { payload: 'Invalid', parents: ['non-existent'] }
      ];
      
      await expect(dag.addVertices(invalidInputs)).rejects.toThrow();
      
      // Verify none were created
      const tips = await dag.getTips();
      expect(tips).toHaveLength(0);
    });
  });
  
  describe('DAG Statistics', () => {
    it('should provide accurate vertex count', async () => {
      expect(await dag.getVertexCount()).toBe(0);
      
      for (let i = 0; i < 10; i++) {
        await dag.addVertex({ payload: `V${i}`, parents: [] });
      }
      
      expect(await dag.getVertexCount()).toBe(10);
    });
    
    it('should calculate DAG depth', async () => {
      const v1 = await dag.addVertex({ payload: 'V1', parents: [] });
      const v2 = await dag.addVertex({ payload: 'V2', parents: [v1] });
      const v3 = await dag.addVertex({ payload: 'V3', parents: [v2] });
      const v4 = await dag.addVertex({ payload: 'V4', parents: [v3] });
      
      expect(await dag.getDepth()).toBe(3); // 0-indexed
    });
    
    it('should provide branching factor statistics', async () => {
      const root = await dag.addVertex({ payload: 'Root', parents: [] });
      
      // Create vertices with different numbers of children
      for (let i = 0; i < 5; i++) {
        await dag.addVertex({ payload: `Child${i}`, parents: [root] });
      }
      
      const stats = await dag.getBranchingStats();
      expect(stats.average).toBeGreaterThan(0);
      expect(stats.max).toBe(5);
      expect(stats.distribution[5]).toBe(1); // One vertex has 5 children
    });
  });
});