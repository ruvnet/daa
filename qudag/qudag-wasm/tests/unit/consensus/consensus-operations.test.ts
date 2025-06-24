import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import type { 
  QuDAG,
  ConsensusManager,
  VertexId,
  ConsensusStatus,
  ConfidenceInfo,
  VotingRecord
} from '@/types';

describe('Consensus Operations', () => {
  let dag: QuDAG;
  let consensus: ConsensusManager;
  let vertices: VertexId[] = [];
  
  beforeEach(async () => {
    const { QuDAG } = await import('@/dag');
    dag = await QuDAG.create({
      consensus: {
        enabled: true,
        algorithm: 'avalanche',
        parameters: {
          k: 10,          // Sample size
          alpha: 8,       // Quorum size
          beta1: 11,      // First confidence threshold
          beta2: 150      // Second confidence threshold
        }
      }
    });
    
    consensus = dag.consensus;
    
    // Create test vertices
    const genesis = await dag.addVertex({ payload: 'Genesis', parents: [] });
    vertices = [genesis];
    
    for (let i = 0; i < 10; i++) {
      const v = await dag.addVertex({
        payload: `Transaction ${i}`,
        parents: [vertices[vertices.length - 1]]
      });
      vertices.push(v);
    }
  });
  
  afterEach(() => {
    dag.dispose();
  });
  
  describe('Voting Operations', () => {
    it('should record votes for vertices', async () => {
      const vertex = vertices[1];
      
      await consensus.vote(vertex, true);
      
      const record = await consensus.getVotingRecord(vertex);
      expect(record.rounds).toHaveLength(1);
      expect(record.rounds[0].votes.positive).toBe(1);
      expect(record.rounds[0].votes.negative).toBe(0);
    });
    
    it('should handle batch voting', async () => {
      const votes = new Map<VertexId, boolean>([
        [vertices[1], true],
        [vertices[2], true],
        [vertices[3], false],
        [vertices[4], true]
      ]);
      
      await consensus.batchVote(votes);
      
      for (const [vertex, vote] of votes) {
        const record = await consensus.getVotingRecord(vertex);
        if (vote) {
          expect(record.rounds[0].votes.positive).toBeGreaterThan(0);
        } else {
          expect(record.rounds[0].votes.negative).toBeGreaterThan(0);
        }
      }
    });
    
    it('should prevent duplicate votes in same round', async () => {
      const vertex = vertices[1];
      
      await consensus.vote(vertex, true);
      await expect(consensus.vote(vertex, false))
        .rejects.toThrow('Already voted in current round');
    });
    
    it('should allow voting in new rounds', async () => {
      const vertex = vertices[1];
      
      // First round
      await consensus.vote(vertex, true);
      
      // Advance round
      await consensus.advanceRound();
      
      // Second round - should be allowed
      await consensus.vote(vertex, false);
      
      const record = await consensus.getVotingRecord(vertex);
      expect(record.rounds).toHaveLength(2);
      expect(record.rounds[0].votes.positive).toBe(1);
      expect(record.rounds[1].votes.negative).toBe(1);
    });
  });
  
  describe('Confidence Tracking', () => {
    it('should calculate confidence based on votes', async () => {
      const vertex = vertices[1];
      
      // Simulate multiple positive votes
      for (let i = 0; i < 8; i++) {
        await consensus.simulateVote(vertex, true, `peer-${i}`);
      }
      
      const confidence = await consensus.getConfidence(vertex);
      expect(confidence.value).toBeGreaterThan(0.5);
      expect(confidence.votes.positive).toBe(8);
      expect(confidence.votes.total).toBe(8);
    });
    
    it('should track confidence history', async () => {
      const vertex = vertices[1];
      
      // Generate confidence changes over time
      for (let round = 0; round < 5; round++) {
        for (let i = 0; i < 10; i++) {
          await consensus.simulateVote(vertex, Math.random() > 0.3, `peer-${i}`);
        }
        await consensus.advanceRound();
      }
      
      const history = await consensus.getConfidenceHistory(vertex);
      expect(history).toHaveLength(5);
      
      // Verify history is chronological
      for (let i = 1; i < history.length; i++) {
        expect(history[i].timestamp).toBeGreaterThan(history[i-1].timestamp);
      }
    });
    
    it('should update consensus status based on confidence', async () => {
      const vertex = vertices[1];
      
      // Initially unknown
      let status = await consensus.getConsensusStatus(vertex);
      expect(status).toBe('unknown');
      
      // Add votes to increase confidence
      for (let round = 0; round < 20; round++) {
        for (let i = 0; i < 10; i++) {
          await consensus.simulateVote(vertex, true, `peer-${i}`);
        }
        await consensus.advanceRound();
      }
      
      status = await consensus.getConsensusStatus(vertex);
      expect(status).toBe('accepted');
    });
  });
  
  describe('Finality Detection', () => {
    it('should detect when vertex reaches finality', async () => {
      const vertex = vertices[1];
      const finalityPromise = consensus.awaitFinality(vertex, 5000);
      
      // Simulate strong consensus
      for (let round = 0; round < 30; round++) {
        for (let i = 0; i < 10; i++) {
          await consensus.simulateVote(vertex, true, `peer-${i}`);
        }
        await consensus.advanceRound();
      }
      
      await expect(finalityPromise).resolves.toBeUndefined();
      
      const status = await consensus.getConsensusStatus(vertex);
      expect(status).toBe('finalized');
    });
    
    it('should timeout if finality not reached', async () => {
      const vertex = vertices[1];
      
      await expect(consensus.awaitFinality(vertex, 100))
        .rejects.toThrow('Timeout waiting for finality');
    });
    
    it('should notify finality observers', async () => {
      const observer = vi.fn();
      const unsubscribe = consensus.onFinality(observer);
      
      const vertex = vertices[1];
      
      // Simulate reaching finality
      for (let round = 0; round < 30; round++) {
        for (let i = 0; i < 10; i++) {
          await consensus.simulateVote(vertex, true, `peer-${i}`);
        }
        await consensus.advanceRound();
      }
      
      expect(observer).toHaveBeenCalledWith(vertex);
      
      unsubscribe();
    });
  });
  
  describe('Conflict Resolution', () => {
    it('should detect conflicting vertices', async () => {
      // Create conflicting transactions (double-spend scenario)
      const conflict1 = await dag.addVertex({
        payload: { type: 'transfer', from: 'A', to: 'B', amount: 100 },
        parents: [vertices[0]],
        metadata: { conflictSet: 'transfer-1' }
      });
      
      const conflict2 = await dag.addVertex({
        payload: { type: 'transfer', from: 'A', to: 'C', amount: 100 },
        parents: [vertices[0]],
        metadata: { conflictSet: 'transfer-1' }
      });
      
      const conflicts = await consensus.getConflictSet(conflict1);
      expect(conflicts).toContain(conflict2);
      
      const isConflicting = await consensus.areConflicting(conflict1, conflict2);
      expect(isConflicting).toBe(true);
    });
    
    it('should resolve conflicts based on confidence', async () => {
      const conflict1 = await dag.addVertex({
        payload: 'Option 1',
        parents: [vertices[0]],
        metadata: { conflictSet: 'choice' }
      });
      
      const conflict2 = await dag.addVertex({
        payload: 'Option 2',
        parents: [vertices[0]],
        metadata: { conflictSet: 'choice' }
      });
      
      // Vote more for conflict1
      for (let i = 0; i < 8; i++) {
        await consensus.simulateVote(conflict1, true, `peer-${i}`);
      }
      
      for (let i = 0; i < 3; i++) {
        await consensus.simulateVote(conflict2, true, `peer-${i}`);
      }
      
      const winner = await consensus.resolveConflict([conflict1, conflict2]);
      expect(winner).toBe(conflict1);
      
      // Verify loser is rejected
      const status2 = await consensus.getConsensusStatus(conflict2);
      expect(status2).toBe('rejected');
    });
  });
  
  describe('Real-time Monitoring', () => {
    it('should stream confidence updates', async () => {
      const vertex = vertices[1];
      const updates: ConfidenceInfo[] = [];
      
      const unsubscribe = consensus.watchConfidence(vertex, (confidence) => {
        updates.push(confidence);
      });
      
      // Generate confidence changes
      for (let round = 0; round < 3; round++) {
        for (let i = 0; i < 5; i++) {
          await consensus.simulateVote(vertex, true, `peer-${i}`);
        }
        await consensus.advanceRound();
      }
      
      expect(updates.length).toBeGreaterThan(0);
      expect(updates[updates.length - 1].value)
        .toBeGreaterThan(updates[0].value);
      
      unsubscribe();
    });
    
    it('should provide consensus statistics', async () => {
      // Generate various consensus states
      for (let i = 0; i < 5; i++) {
        for (let j = 0; j < 10; j++) {
          await consensus.simulateVote(vertices[i], true, `peer-${j}`);
        }
      }
      
      const stats = await consensus.getStats();
      
      expect(stats.totalVertices).toBeGreaterThanOrEqual(5);
      expect(stats.consensusReached).toBeGreaterThanOrEqual(0);
      expect(stats.pending).toBeGreaterThanOrEqual(0);
      expect(stats.averageConfidence).toBeGreaterThan(0);
      expect(stats.averageRounds).toBeGreaterThan(0);
    });
  });
  
  describe('Query Operations', () => {
    it('should get vertices by consensus status', async () => {
      // Create vertices with different statuses
      for (let i = 0; i < 5; i++) {
        for (let j = 0; j < 10; j++) {
          await consensus.simulateVote(vertices[i], true, `peer-${j}`);
        }
        if (i < 3) {
          // Make first 3 reach higher confidence
          for (let round = 0; round < 20; round++) {
            await consensus.advanceRound();
            for (let j = 0; j < 10; j++) {
              await consensus.simulateVote(vertices[i], true, `peer-${j}`);
            }
          }
        }
      }
      
      const accepted = await consensus.getVerticesByStatus('accepted');
      expect(accepted.length).toBeGreaterThan(0);
      
      const unknown = await consensus.getVerticesByStatus('unknown');
      expect(unknown.length).toBeGreaterThan(0);
    });
    
    it('should get preferred vertices', async () => {
      // Create competing vertices
      const option1 = await dag.addVertex({
        payload: 'Option 1',
        parents: [vertices[0]]
      });
      
      const option2 = await dag.addVertex({
        payload: 'Option 2',
        parents: [vertices[0]]
      });
      
      // Vote more for option1
      for (let i = 0; i < 8; i++) {
        await consensus.simulateVote(option1, true, `peer-${i}`);
      }
      
      for (let i = 0; i < 3; i++) {
        await consensus.simulateVote(option2, true, `peer-${i}`);
      }
      
      const preferred = await consensus.getPreferred([option1, option2]);
      expect(preferred).toBe(option1);
    });
  });
  
  describe('Performance and Optimization', () => {
    it('should handle large-scale voting efficiently', async () => {
      const startTime = performance.now();
      
      // Simulate 1000 votes across 100 vertices
      const votePromises = [];
      for (let i = 0; i < 100 i++) {
        const vertex = vertices[i % vertices.length];
        for (let j = 0; j < 10; j++) {
          votePromises.push(
            consensus.simulateVote(vertex, Math.random() > 0.5, `peer-${j}`)
          );
        }
      }
      
      await Promise.all(votePromises);
      
      const duration = performance.now() - startTime;
      expect(duration).toBeLessThan(1000); // Should complete within 1 second
    });
    
    it('should cache confidence calculations', async () => {
      const vertex = vertices[1];
      
      // First call - calculates
      const [conf1, time1] = await testUtils.measureTime(
        () => consensus.getConfidence(vertex)
      );
      
      // Second call - should be cached
      const [conf2, time2] = await testUtils.measureTime(
        () => consensus.getConfidence(vertex)
      );
      
      expect(conf1).toEqual(conf2);
      expect(time2).toBeLessThan(time1 * 0.1); // 10x faster from cache
    });
  });
  
  describe('Error Handling', () => {
    it('should handle voting on non-existent vertex', async () => {
      const fakeId = 'f'.repeat(64);
      
      await expect(consensus.vote(fakeId, true))
        .rejects.toThrow('Vertex not found');
    });
    
    it('should handle invalid consensus parameters', async () => {
      await expect(consensus.configure({
        k: -1,
        alpha: 10
      })).rejects.toThrow('Invalid consensus parameters');
    });
    
    it('should recover from consensus failures', async () => {
      const vertex = vertices[1];
      
      // Simulate network partition
      consensus.simulateNetworkPartition();
      
      // Votes during partition should be queued
      await consensus.vote(vertex, true);
      
      // Heal partition
      consensus.healNetworkPartition();
      
      // Votes should be processed
      const record = await consensus.getVotingRecord(vertex);
      expect(record.rounds[0].votes.positive).toBeGreaterThan(0);
    });
  });
});