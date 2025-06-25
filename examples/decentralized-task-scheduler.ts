/**
 * Decentralized Task Scheduler with Consensus-Based Leader Election
 * Implements Raft consensus, job partitioning, load balancing, and Byzantine fault tolerance
 */

// Core types and interfaces
interface Node {
  id: string;
  address: string;
  publicKey: string;
  reputation: number;
  capacity: NodeCapacity;
  status: NodeStatus;
  lastHeartbeat: number;
  joinedAt: number;
}

interface NodeCapacity {
  cpu: number;
  memory: number;
  bandwidth: number;
  currentLoad: number;
}

enum NodeStatus {
  ACTIVE = 'ACTIVE',
  INACTIVE = 'INACTIVE',
  SUSPECTED = 'SUSPECTED',
  BLACKLISTED = 'BLACKLISTED'
}

interface Task {
  id: string;
  type: TaskType;
  priority: number;
  requirements: TaskRequirements;
  payload: any;
  createdAt: number;
  deadline?: number;
  assignedTo?: string;
  status: TaskStatus;
  validationRequirement?: ValidationRequirement;
}

interface TaskRequirements {
  minCpu: number;
  minMemory: number;
  estimatedDuration: number;
  trustLevel?: number;
}

interface ValidationRequirement {
  validatorCount: number;
  consensusThreshold: number;
  validationTimeout: number;
}

enum TaskType {
  COMPUTE = 'COMPUTE',
  STORAGE = 'STORAGE',
  VALIDATION = 'VALIDATION',
  CONSENSUS = 'CONSENSUS'
}

enum TaskStatus {
  PENDING = 'PENDING',
  ASSIGNED = 'ASSIGNED',
  EXECUTING = 'EXECUTING',
  VALIDATING = 'VALIDATING',
  COMPLETED = 'COMPLETED',
  FAILED = 'FAILED'
}

// Raft consensus implementation for leader election
class RaftConsensus {
  private nodeId: string;
  private currentTerm: number = 0;
  private votedFor: string | null = null;
  private state: RaftState = RaftState.FOLLOWER;
  private leader: string | null = null;
  private electionTimeout: number;
  private heartbeatInterval: number = 150;
  private votes: Map<string, boolean> = new Map();
  private log: RaftLogEntry[] = [];
  private commitIndex: number = 0;
  private lastApplied: number = 0;

  constructor(nodeId: string) {
    this.nodeId = nodeId;
    this.electionTimeout = this.getRandomElectionTimeout();
  }

  private getRandomElectionTimeout(): number {
    return 300 + Math.floor(Math.random() * 150); // 300-450ms
  }

  public startElection(nodes: string[]): void {
    this.currentTerm++;
    this.state = RaftState.CANDIDATE;
    this.votedFor = this.nodeId;
    this.votes.clear();
    this.votes.set(this.nodeId, true);

    const voteRequest: VoteRequest = {
      term: this.currentTerm,
      candidateId: this.nodeId,
      lastLogIndex: this.log.length - 1,
      lastLogTerm: this.log.length > 0 ? this.log[this.log.length - 1].term : 0
    };

    // Request votes from other nodes
    nodes.forEach(node => {
      if (node !== this.nodeId) {
        this.requestVote(node, voteRequest);
      }
    });
  }

  private requestVote(node: string, request: VoteRequest): void {
    // Simulate vote request - in real implementation, this would be a network call
    // For demonstration, we'll simulate positive votes for majority
    setTimeout(() => {
      this.handleVoteResponse(node, {
        term: request.term,
        voteGranted: Math.random() > 0.3 // 70% chance of vote
      });
    }, Math.random() * 50);
  }

  private handleVoteResponse(node: string, response: VoteResponse): void {
    if (response.term > this.currentTerm) {
      this.currentTerm = response.term;
      this.state = RaftState.FOLLOWER;
      this.votedFor = null;
      return;
    }

    if (this.state === RaftState.CANDIDATE && response.voteGranted) {
      this.votes.set(node, true);
      if (this.votes.size > Math.floor(this.getNodeCount() / 2)) {
        this.becomeLeader();
      }
    }
  }

  private becomeLeader(): void {
    this.state = RaftState.LEADER;
    this.leader = this.nodeId;
    console.log(`Node ${this.nodeId} became leader for term ${this.currentTerm}`);
    this.sendHeartbeats();
  }

  private sendHeartbeats(): void {
    // Leader sends periodic heartbeats to maintain authority
    setInterval(() => {
      if (this.state === RaftState.LEADER) {
        // Send heartbeat to all nodes
        this.broadcastAppendEntries();
      }
    }, this.heartbeatInterval);
  }

  private broadcastAppendEntries(): void {
    // Broadcast append entries (heartbeat) to all nodes
    const appendEntries: AppendEntries = {
      term: this.currentTerm,
      leaderId: this.nodeId,
      prevLogIndex: this.log.length - 1,
      prevLogTerm: this.log.length > 0 ? this.log[this.log.length - 1].term : 0,
      entries: [],
      leaderCommit: this.commitIndex
    };
    // In real implementation, send to all nodes
  }

  private getNodeCount(): number {
    return 5; // Placeholder - in real implementation, get from membership manager
  }

  public isLeader(): boolean {
    return this.state === RaftState.LEADER;
  }

  public getLeader(): string | null {
    return this.leader;
  }
}

enum RaftState {
  FOLLOWER = 'FOLLOWER',
  CANDIDATE = 'CANDIDATE',
  LEADER = 'LEADER'
}

interface RaftLogEntry {
  term: number;
  command: any;
  index: number;
}

interface VoteRequest {
  term: number;
  candidateId: string;
  lastLogIndex: number;
  lastLogTerm: number;
}

interface VoteResponse {
  term: number;
  voteGranted: boolean;
}

interface AppendEntries {
  term: number;
  leaderId: string;
  prevLogIndex: number;
  prevLogTerm: number;
  entries: RaftLogEntry[];
  leaderCommit: number;
}

// Job Partitioning Algorithms
class JobPartitioner {
  private partitionStrategies: Map<string, PartitionStrategy>;

  constructor() {
    this.partitionStrategies = new Map([
      ['hash', new HashPartitionStrategy()],
      ['range', new RangePartitionStrategy()],
      ['consistent', new ConsistentHashingStrategy()],
      ['workload', new WorkloadAwareStrategy()]
    ]);
  }

  public partitionTasks(
    tasks: Task[],
    nodes: Node[],
    strategy: string = 'workload'
  ): Map<string, Task[]> {
    const partitioner = this.partitionStrategies.get(strategy);
    if (!partitioner) {
      throw new Error(`Unknown partition strategy: ${strategy}`);
    }
    return partitioner.partition(tasks, nodes);
  }
}

interface PartitionStrategy {
  partition(tasks: Task[], nodes: Node[]): Map<string, Task[]>;
}

class HashPartitionStrategy implements PartitionStrategy {
  partition(tasks: Task[], nodes: Node[]): Map<string, Task[]> {
    const partitions = new Map<string, Task[]>();
    const activeNodes = nodes.filter(n => n.status === NodeStatus.ACTIVE);
    
    tasks.forEach(task => {
      const hash = this.hashTask(task);
      const nodeIndex = hash % activeNodes.length;
      const node = activeNodes[nodeIndex];
      
      if (!partitions.has(node.id)) {
        partitions.set(node.id, []);
      }
      partitions.get(node.id)!.push(task);
    });
    
    return partitions;
  }

  private hashTask(task: Task): number {
    let hash = 0;
    for (let i = 0; i < task.id.length; i++) {
      hash = ((hash << 5) - hash) + task.id.charCodeAt(i);
      hash = hash & hash; // Convert to 32bit integer
    }
    return Math.abs(hash);
  }
}

class RangePartitionStrategy implements PartitionStrategy {
  partition(tasks: Task[], nodes: Node[]): Map<string, Task[]> {
    const partitions = new Map<string, Task[]>();
    const activeNodes = nodes.filter(n => n.status === NodeStatus.ACTIVE);
    const sortedTasks = [...tasks].sort((a, b) => a.priority - b.priority);
    
    const tasksPerNode = Math.ceil(sortedTasks.length / activeNodes.length);
    
    activeNodes.forEach((node, index) => {
      const start = index * tasksPerNode;
      const end = Math.min(start + tasksPerNode, sortedTasks.length);
      const nodeTasks = sortedTasks.slice(start, end);
      
      if (nodeTasks.length > 0) {
        partitions.set(node.id, nodeTasks);
      }
    });
    
    return partitions;
  }
}

class ConsistentHashingStrategy implements PartitionStrategy {
  private virtualNodes: number = 150;
  private ring: Map<number, string> = new Map();

  partition(tasks: Task[], nodes: Node[]): Map<string, Task[]> {
    this.buildHashRing(nodes);
    const partitions = new Map<string, Task[]>();
    
    tasks.forEach(task => {
      const node = this.getNodeForTask(task);
      if (!partitions.has(node)) {
        partitions.set(node, []);
      }
      partitions.get(node)!.push(task);
    });
    
    return partitions;
  }

  private buildHashRing(nodes: Node[]): void {
    this.ring.clear();
    const activeNodes = nodes.filter(n => n.status === NodeStatus.ACTIVE);
    
    activeNodes.forEach(node => {
      for (let i = 0; i < this.virtualNodes; i++) {
        const hash = this.hash(`${node.id}-${i}`);
        this.ring.set(hash, node.id);
      }
    });
  }

  private getNodeForTask(task: Task): string {
    const taskHash = this.hash(task.id);
    const ringKeys = Array.from(this.ring.keys()).sort((a, b) => a - b);
    
    for (const key of ringKeys) {
      if (key >= taskHash) {
        return this.ring.get(key)!;
      }
    }
    
    return this.ring.get(ringKeys[0])!;
  }

  private hash(key: string): number {
    let hash = 0;
    for (let i = 0; i < key.length; i++) {
      hash = ((hash << 5) - hash) + key.charCodeAt(i);
      hash = hash & hash;
    }
    return Math.abs(hash);
  }
}

class WorkloadAwareStrategy implements PartitionStrategy {
  partition(tasks: Task[], nodes: Node[]): Map<string, Task[]> {
    const partitions = new Map<string, Task[]>();
    const activeNodes = nodes
      .filter(n => n.status === NodeStatus.ACTIVE)
      .sort((a, b) => this.getAvailableCapacity(a) - this.getAvailableCapacity(b));
    
    const sortedTasks = [...tasks].sort((a, b) => b.priority - a.priority);
    
    sortedTasks.forEach(task => {
      const suitableNode = this.findSuitableNode(task, activeNodes);
      if (suitableNode) {
        if (!partitions.has(suitableNode.id)) {
          partitions.set(suitableNode.id, []);
        }
        partitions.get(suitableNode.id)!.push(task);
        this.updateNodeLoad(suitableNode, task);
      }
    });
    
    return partitions;
  }

  private getAvailableCapacity(node: Node): number {
    return 1 - node.capacity.currentLoad;
  }

  private findSuitableNode(task: Task, nodes: Node[]): Node | null {
    return nodes.find(node => 
      node.capacity.cpu >= task.requirements.minCpu &&
      node.capacity.memory >= task.requirements.minMemory &&
      node.capacity.currentLoad < 0.8 &&
      (!task.requirements.trustLevel || node.reputation >= task.requirements.trustLevel)
    ) || null;
  }

  private updateNodeLoad(node: Node, task: Task): void {
    const estimatedLoad = task.requirements.minCpu / node.capacity.cpu;
    node.capacity.currentLoad = Math.min(1, node.capacity.currentLoad + estimatedLoad);
  }
}

// Work Assignment Protocol
class WorkAssignmentProtocol {
  private assignments: Map<string, Assignment> = new Map();
  private nodeCapabilities: Map<string, NodeCapability> = new Map();

  public assignWork(
    task: Task,
    node: Node,
    validationRequired: boolean = false
  ): Assignment {
    const assignment: Assignment = {
      id: this.generateAssignmentId(),
      taskId: task.id,
      nodeId: node.id,
      assignedAt: Date.now(),
      deadline: task.deadline || Date.now() + task.requirements.estimatedDuration,
      status: AssignmentStatus.ASSIGNED,
      validationRequired,
      validators: validationRequired ? this.selectValidators(task, node) : []
    };

    this.assignments.set(assignment.id, assignment);
    this.updateNodeCapability(node, task);
    
    return assignment;
  }

  private selectValidators(task: Task, assignedNode: Node): string[] {
    const validatorCount = task.validationRequirement?.validatorCount || 3;
    // Select validators excluding the assigned node
    // In real implementation, select based on reputation and availability
    return [];
  }

  private updateNodeCapability(node: Node, task: Task): void {
    const capability: NodeCapability = {
      nodeId: node.id,
      availableCpu: node.capacity.cpu - task.requirements.minCpu,
      availableMemory: node.capacity.memory - task.requirements.minMemory,
      taskTypes: [task.type],
      successRate: 0.95, // Placeholder
      avgCompletionTime: task.requirements.estimatedDuration
    };
    
    this.nodeCapabilities.set(node.id, capability);
  }

  public getAssignment(assignmentId: string): Assignment | undefined {
    return this.assignments.get(assignmentId);
  }

  public updateAssignmentStatus(assignmentId: string, status: AssignmentStatus): void {
    const assignment = this.assignments.get(assignmentId);
    if (assignment) {
      assignment.status = status;
      assignment.updatedAt = Date.now();
    }
  }

  private generateAssignmentId(): string {
    return `assign-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}

interface Assignment {
  id: string;
  taskId: string;
  nodeId: string;
  assignedAt: number;
  deadline: number;
  status: AssignmentStatus;
  validationRequired: boolean;
  validators: string[];
  result?: any;
  updatedAt?: number;
}

interface NodeCapability {
  nodeId: string;
  availableCpu: number;
  availableMemory: number;
  taskTypes: TaskType[];
  successRate: number;
  avgCompletionTime: number;
}

enum AssignmentStatus {
  ASSIGNED = 'ASSIGNED',
  IN_PROGRESS = 'IN_PROGRESS',
  COMPLETED = 'COMPLETED',
  FAILED = 'FAILED',
  TIMEOUT = 'TIMEOUT',
  VALIDATING = 'VALIDATING',
  VALIDATED = 'VALIDATED',
  REJECTED = 'REJECTED'
}

// Load Balancer
class LoadBalancer {
  private strategies: Map<string, LoadBalancingStrategy>;
  private currentStrategy: string = 'weighted-round-robin';
  private metrics: LoadMetrics;

  constructor() {
    this.strategies = new Map([
      ['round-robin', new RoundRobinStrategy()],
      ['weighted-round-robin', new WeightedRoundRobinStrategy()],
      ['least-connections', new LeastConnectionsStrategy()],
      ['response-time', new ResponseTimeStrategy()],
      ['adaptive', new AdaptiveLoadBalancingStrategy()]
    ]);
    this.metrics = new LoadMetrics();
  }

  public selectNode(nodes: Node[], task: Task): Node | null {
    const strategy = this.strategies.get(this.currentStrategy);
    if (!strategy) {
      throw new Error(`Unknown load balancing strategy: ${this.currentStrategy}`);
    }

    const eligibleNodes = this.filterEligibleNodes(nodes, task);
    if (eligibleNodes.length === 0) {
      return null;
    }

    const selectedNode = strategy.selectNode(eligibleNodes, task, this.metrics);
    this.metrics.recordSelection(selectedNode.id);
    
    return selectedNode;
  }

  private filterEligibleNodes(nodes: Node[], task: Task): Node[] {
    return nodes.filter(node =>
      node.status === NodeStatus.ACTIVE &&
      node.capacity.cpu >= task.requirements.minCpu &&
      node.capacity.memory >= task.requirements.minMemory &&
      node.capacity.currentLoad < 0.9 &&
      (!task.requirements.trustLevel || node.reputation >= task.requirements.trustLevel)
    );
  }

  public updateMetrics(nodeId: string, responseTime: number, success: boolean): void {
    this.metrics.updateNodeMetrics(nodeId, responseTime, success);
  }

  public setStrategy(strategy: string): void {
    if (this.strategies.has(strategy)) {
      this.currentStrategy = strategy;
    }
  }
}

interface LoadBalancingStrategy {
  selectNode(nodes: Node[], task: Task, metrics: LoadMetrics): Node;
}

class RoundRobinStrategy implements LoadBalancingStrategy {
  private currentIndex: number = 0;

  selectNode(nodes: Node[], task: Task, metrics: LoadMetrics): Node {
    const node = nodes[this.currentIndex % nodes.length];
    this.currentIndex++;
    return node;
  }
}

class WeightedRoundRobinStrategy implements LoadBalancingStrategy {
  private weightedNodes: Array<{ node: Node; weight: number }> = [];
  private currentWeight: number = 0;

  selectNode(nodes: Node[], task: Task, metrics: LoadMetrics): Node {
    this.updateWeights(nodes);
    
    let totalWeight = this.weightedNodes.reduce((sum, wn) => sum + wn.weight, 0);
    let random = Math.random() * totalWeight;
    
    for (const weightedNode of this.weightedNodes) {
      random -= weightedNode.weight;
      if (random <= 0) {
        return weightedNode.node;
      }
    }
    
    return this.weightedNodes[0].node;
  }

  private updateWeights(nodes: Node[]): void {
    this.weightedNodes = nodes.map(node => ({
      node,
      weight: this.calculateWeight(node)
    }));
  }

  private calculateWeight(node: Node): number {
    const capacityWeight = (1 - node.capacity.currentLoad) * 100;
    const reputationWeight = node.reputation * 10;
    return capacityWeight + reputationWeight;
  }
}

class LeastConnectionsStrategy implements LoadBalancingStrategy {
  selectNode(nodes: Node[], task: Task, metrics: LoadMetrics): Node {
    return nodes.reduce((least, node) => {
      const nodeConnections = metrics.getActiveConnections(node.id);
      const leastConnections = metrics.getActiveConnections(least.id);
      return nodeConnections < leastConnections ? node : least;
    });
  }
}

class ResponseTimeStrategy implements LoadBalancingStrategy {
  selectNode(nodes: Node[], task: Task, metrics: LoadMetrics): Node {
    return nodes.reduce((fastest, node) => {
      const nodeResponseTime = metrics.getAverageResponseTime(node.id);
      const fastestResponseTime = metrics.getAverageResponseTime(fastest.id);
      return nodeResponseTime < fastestResponseTime ? node : fastest;
    });
  }
}

class AdaptiveLoadBalancingStrategy implements LoadBalancingStrategy {
  private strategies: LoadBalancingStrategy[];
  private weights: number[];

  constructor() {
    this.strategies = [
      new WeightedRoundRobinStrategy(),
      new LeastConnectionsStrategy(),
      new ResponseTimeStrategy()
    ];
    this.weights = [0.4, 0.3, 0.3];
  }

  selectNode(nodes: Node[], task: Task, metrics: LoadMetrics): Node {
    // Adaptive strategy combines multiple strategies with weights
    const scores = new Map<string, number>();
    
    nodes.forEach(node => scores.set(node.id, 0));
    
    this.strategies.forEach((strategy, index) => {
      const selectedNode = strategy.selectNode(nodes, task, metrics);
      const currentScore = scores.get(selectedNode.id) || 0;
      scores.set(selectedNode.id, currentScore + this.weights[index]);
    });
    
    let bestNode = nodes[0];
    let bestScore = 0;
    
    scores.forEach((score, nodeId) => {
      if (score > bestScore) {
        bestScore = score;
        bestNode = nodes.find(n => n.id === nodeId)!;
      }
    });
    
    return bestNode;
  }
}

class LoadMetrics {
  private nodeMetrics: Map<string, NodeMetric> = new Map();

  recordSelection(nodeId: string): void {
    const metric = this.getOrCreateMetric(nodeId);
    metric.activeConnections++;
    metric.totalRequests++;
  }

  updateNodeMetrics(nodeId: string, responseTime: number, success: boolean): void {
    const metric = this.getOrCreateMetric(nodeId);
    metric.activeConnections = Math.max(0, metric.activeConnections - 1);
    metric.responseTimes.push(responseTime);
    if (metric.responseTimes.length > 100) {
      metric.responseTimes.shift();
    }
    if (success) {
      metric.successfulRequests++;
    } else {
      metric.failedRequests++;
    }
  }

  getActiveConnections(nodeId: string): number {
    return this.getOrCreateMetric(nodeId).activeConnections;
  }

  getAverageResponseTime(nodeId: string): number {
    const metric = this.getOrCreateMetric(nodeId);
    if (metric.responseTimes.length === 0) return 0;
    const sum = metric.responseTimes.reduce((a, b) => a + b, 0);
    return sum / metric.responseTimes.length;
  }

  private getOrCreateMetric(nodeId: string): NodeMetric {
    if (!this.nodeMetrics.has(nodeId)) {
      this.nodeMetrics.set(nodeId, {
        activeConnections: 0,
        totalRequests: 0,
        successfulRequests: 0,
        failedRequests: 0,
        responseTimes: []
      });
    }
    return this.nodeMetrics.get(nodeId)!;
  }
}

interface NodeMetric {
  activeConnections: number;
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  responseTimes: number[];
}

// Validator Task Assignment with Byzantine Fault Tolerance
class ValidatorAssignment {
  private validatorPool: Map<string, Validator> = new Map();
  private validationTasks: Map<string, ValidationTask> = new Map();
  private byzantineThreshold: number = 0.33; // Tolerate up to 33% Byzantine nodes

  public assignValidators(
    task: Task,
    result: any,
    validatorCount: number = 3
  ): ValidationTask {
    const validators = this.selectValidators(task, validatorCount);
    const validationTask: ValidationTask = {
      id: this.generateValidationId(),
      taskId: task.id,
      result,
      validators: validators.map(v => v.id),
      validationResults: new Map(),
      consensusThreshold: task.validationRequirement?.consensusThreshold || 0.67,
      startTime: Date.now(),
      timeout: task.validationRequirement?.validationTimeout || 30000,
      status: ValidationStatus.PENDING
    };

    this.validationTasks.set(validationTask.id, validationTask);
    
    // Notify validators
    validators.forEach(validator => {
      this.notifyValidator(validator, validationTask);
    });

    return validationTask;
  }

  private selectValidators(task: Task, count: number): Validator[] {
    const eligibleValidators = Array.from(this.validatorPool.values())
      .filter(v => 
        v.status === ValidatorStatus.AVAILABLE &&
        v.reputation > 0.7 &&
        v.specializations.includes(task.type)
      )
      .sort((a, b) => b.reputation - a.reputation);

    return eligibleValidators.slice(0, count);
  }

  private notifyValidator(validator: Validator, validationTask: ValidationTask): void {
    // In real implementation, send validation request to validator
    validator.status = ValidatorStatus.VALIDATING;
    validator.currentTask = validationTask.id;
  }

  public submitValidationResult(
    validationTaskId: string,
    validatorId: string,
    result: ValidationResult
  ): void {
    const validationTask = this.validationTasks.get(validationTaskId);
    if (!validationTask) return;

    validationTask.validationResults.set(validatorId, result);
    
    // Check if we have enough results for consensus
    if (validationTask.validationResults.size >= validationTask.validators.length) {
      this.evaluateConsensus(validationTask);
    }
  }

  private evaluateConsensus(validationTask: ValidationTask): void {
    const results = Array.from(validationTask.validationResults.values());
    const approvals = results.filter(r => r.approved).length;
    const totalValidators = validationTask.validators.length;
    
    // Byzantine Fault Tolerant consensus
    const byzantineValidators = Math.floor(totalValidators * this.byzantineThreshold);
    const requiredApprovals = Math.ceil((totalValidators - byzantineValidators) * validationTask.consensusThreshold);
    
    if (approvals >= requiredApprovals) {
      validationTask.status = ValidationStatus.APPROVED;
      this.updateValidatorReputations(validationTask, true);
    } else if (totalValidators - approvals < requiredApprovals) {
      validationTask.status = ValidationStatus.REJECTED;
      this.updateValidatorReputations(validationTask, false);
    }
    
    // Release validators
    validationTask.validators.forEach(validatorId => {
      const validator = this.validatorPool.get(validatorId);
      if (validator) {
        validator.status = ValidatorStatus.AVAILABLE;
        validator.currentTask = null;
      }
    });
  }

  private updateValidatorReputations(validationTask: ValidationTask, consensus: boolean): void {
    const majorityResult = consensus;
    
    validationTask.validationResults.forEach((result, validatorId) => {
      const validator = this.validatorPool.get(validatorId);
      if (validator) {
        if (result.approved === majorityResult) {
          // Validator agreed with consensus
          validator.reputation = Math.min(1, validator.reputation + 0.01);
          validator.successfulValidations++;
        } else {
          // Validator disagreed with consensus
          validator.reputation = Math.max(0, validator.reputation - 0.05);
          validator.failedValidations++;
        }
      }
    });
  }

  public registerValidator(validator: Validator): void {
    this.validatorPool.set(validator.id, validator);
  }

  private generateValidationId(): string {
    return `val-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}

interface Validator {
  id: string;
  publicKey: string;
  reputation: number;
  specializations: TaskType[];
  status: ValidatorStatus;
  currentTask: string | null;
  successfulValidations: number;
  failedValidations: number;
  joinedAt: number;
}

interface ValidationTask {
  id: string;
  taskId: string;
  result: any;
  validators: string[];
  validationResults: Map<string, ValidationResult>;
  consensusThreshold: number;
  startTime: number;
  timeout: number;
  status: ValidationStatus;
}

interface ValidationResult {
  approved: boolean;
  reason?: string;
  signature: string;
  timestamp: number;
}

enum ValidatorStatus {
  AVAILABLE = 'AVAILABLE',
  VALIDATING = 'VALIDATING',
  OFFLINE = 'OFFLINE'
}

enum ValidationStatus {
  PENDING = 'PENDING',
  APPROVED = 'APPROVED',
  REJECTED = 'REJECTED',
  TIMEOUT = 'TIMEOUT'
}

// Trust and Reputation Management for Untrusted Nodes
class TrustManager {
  private nodeReputations: Map<string, Reputation> = new Map();
  private trustThresholds = {
    minimal: 0.3,
    basic: 0.5,
    standard: 0.7,
    high: 0.9
  };
  private behaviorHistory: Map<string, BehaviorRecord[]> = new Map();

  public evaluateNodeTrust(nodeId: string): TrustLevel {
    const reputation = this.nodeReputations.get(nodeId);
    if (!reputation) return TrustLevel.UNTRUSTED;

    const score = this.calculateTrustScore(reputation);
    
    if (score >= this.trustThresholds.high) return TrustLevel.HIGH;
    if (score >= this.trustThresholds.standard) return TrustLevel.STANDARD;
    if (score >= this.trustThresholds.basic) return TrustLevel.BASIC;
    if (score >= this.trustThresholds.minimal) return TrustLevel.MINIMAL;
    
    return TrustLevel.UNTRUSTED;
  }

  private calculateTrustScore(reputation: Reputation): number {
    const weights = {
      taskCompletion: 0.3,
      validationAccuracy: 0.3,
      availability: 0.2,
      behavior: 0.2
    };

    return (
      reputation.taskCompletionRate * weights.taskCompletion +
      reputation.validationAccuracy * weights.validationAccuracy +
      reputation.availabilityScore * weights.availability +
      reputation.behaviorScore * weights.behavior
    );
  }

  public recordTaskCompletion(nodeId: string, success: boolean, quality: number): void {
    const reputation = this.getOrCreateReputation(nodeId);
    
    if (success) {
      reputation.successfulTasks++;
      reputation.totalQualityScore += quality;
    } else {
      reputation.failedTasks++;
    }
    
    reputation.taskCompletionRate = 
      reputation.successfulTasks / (reputation.successfulTasks + reputation.failedTasks);
    
    this.recordBehavior(nodeId, BehaviorType.TASK_COMPLETION, success ? 1 : -1);
  }

  public recordValidation(nodeId: string, accurate: boolean): void {
    const reputation = this.getOrCreateReputation(nodeId);
    
    if (accurate) {
      reputation.accurateValidations++;
    } else {
      reputation.inaccurateValidations++;
    }
    
    reputation.validationAccuracy = 
      reputation.accurateValidations / (reputation.accurateValidations + reputation.inaccurateValidations);
    
    this.recordBehavior(nodeId, BehaviorType.VALIDATION, accurate ? 1 : -1);
  }

  public recordAvailability(nodeId: string, online: boolean): void {
    const reputation = this.getOrCreateReputation(nodeId);
    const now = Date.now();
    
    if (online) {
      reputation.totalUptime += now - (reputation.lastOnlineTime || now);
    }
    
    reputation.lastOnlineTime = online ? now : null;
    reputation.availabilityScore = 
      reputation.totalUptime / (now - reputation.joinedAt);
  }

  private recordBehavior(nodeId: string, type: BehaviorType, score: number): void {
    const history = this.behaviorHistory.get(nodeId) || [];
    history.push({
      type,
      score,
      timestamp: Date.now()
    });
    
    // Keep only recent history (last 1000 records)
    if (history.length > 1000) {
      history.shift();
    }
    
    this.behaviorHistory.set(nodeId, history);
    
    // Update behavior score
    const reputation = this.nodeReputations.get(nodeId);
    if (reputation) {
      const recentBehaviors = history.slice(-100);
      const avgScore = recentBehaviors.reduce((sum, b) => sum + b.score, 0) / recentBehaviors.length;
      reputation.behaviorScore = Math.max(0, Math.min(1, (avgScore + 1) / 2));
    }
  }

  public blacklistNode(nodeId: string, reason: string): void {
    const reputation = this.getOrCreateReputation(nodeId);
    reputation.blacklisted = true;
    reputation.blacklistReason = reason;
    reputation.blacklistTime = Date.now();
  }

  public isBlacklisted(nodeId: string): boolean {
    const reputation = this.nodeReputations.get(nodeId);
    return reputation?.blacklisted || false;
  }

  private getOrCreateReputation(nodeId: string): Reputation {
    if (!this.nodeReputations.has(nodeId)) {
      this.nodeReputations.set(nodeId, {
        nodeId,
        taskCompletionRate: 0.5,
        validationAccuracy: 0.5,
        availabilityScore: 0.5,
        behaviorScore: 0.5,
        successfulTasks: 0,
        failedTasks: 0,
        accurateValidations: 0,
        inaccurateValidations: 0,
        totalUptime: 0,
        totalQualityScore: 0,
        joinedAt: Date.now(),
        lastOnlineTime: Date.now(),
        blacklisted: false
      });
    }
    return this.nodeReputations.get(nodeId)!;
  }
}

interface Reputation {
  nodeId: string;
  taskCompletionRate: number;
  validationAccuracy: number;
  availabilityScore: number;
  behaviorScore: number;
  successfulTasks: number;
  failedTasks: number;
  accurateValidations: number;
  inaccurateValidations: number;
  totalUptime: number;
  totalQualityScore: number;
  joinedAt: number;
  lastOnlineTime: number | null;
  blacklisted: boolean;
  blacklistReason?: string;
  blacklistTime?: number;
}

interface BehaviorRecord {
  type: BehaviorType;
  score: number;
  timestamp: number;
}

enum BehaviorType {
  TASK_COMPLETION = 'TASK_COMPLETION',
  VALIDATION = 'VALIDATION',
  AVAILABILITY = 'AVAILABILITY',
  PROTOCOL_VIOLATION = 'PROTOCOL_VIOLATION'
}

enum TrustLevel {
  UNTRUSTED = 'UNTRUSTED',
  MINIMAL = 'MINIMAL',
  BASIC = 'BASIC',
  STANDARD = 'STANDARD',
  HIGH = 'HIGH'
}

// Dynamic Membership Management
class MembershipManager {
  private members: Map<string, Member> = new Map();
  private pendingJoins: Map<string, JoinRequest> = new Map();
  private gossipProtocol: GossipProtocol;
  private membershipView: MembershipView;
  private heartbeatInterval: number = 5000;
  private suspicionTimeout: number = 15000;
  private failureTimeout: number = 30000;

  constructor() {
    this.gossipProtocol = new GossipProtocol();
    this.membershipView = new MembershipView();
    this.startHeartbeatMonitoring();
  }

  public requestJoin(node: Node, proof: JoinProof): JoinRequest {
    const request: JoinRequest = {
      id: this.generateRequestId(),
      node,
      proof,
      timestamp: Date.now(),
      status: JoinStatus.PENDING,
      votes: new Map()
    };

    this.pendingJoins.set(request.id, request);
    this.broadcastJoinRequest(request);
    
    return request;
  }

  private broadcastJoinRequest(request: JoinRequest): void {
    // Broadcast join request to all active members for voting
    this.members.forEach(member => {
      if (member.status === MemberStatus.ACTIVE) {
        // In real implementation, send join request for voting
        this.simulateVoteOnJoin(member.nodeId, request.id);
      }
    });
  }

  private simulateVoteOnJoin(voterId: string, requestId: string): void {
    setTimeout(() => {
      const request = this.pendingJoins.get(requestId);
      if (request) {
        // Simulate vote based on proof validation
        const vote = Math.random() > 0.2; // 80% approval rate
        request.votes.set(voterId, vote);
        this.checkJoinConsensus(requestId);
      }
    }, Math.random() * 1000);
  }

  private checkJoinConsensus(requestId: string): void {
    const request = this.pendingJoins.get(requestId);
    if (!request) return;

    const totalMembers = this.members.size;
    const approvals = Array.from(request.votes.values()).filter(v => v).length;
    const requiredApprovals = Math.ceil(totalMembers * 0.51); // Simple majority

    if (approvals >= requiredApprovals) {
      request.status = JoinStatus.APPROVED;
      this.addMember(request.node);
      this.pendingJoins.delete(requestId);
    } else if (request.votes.size >= totalMembers) {
      request.status = JoinStatus.REJECTED;
      this.pendingJoins.delete(requestId);
    }
  }

  private addMember(node: Node): void {
    const member: Member = {
      nodeId: node.id,
      address: node.address,
      publicKey: node.publicKey,
      joinedAt: Date.now(),
      lastHeartbeat: Date.now(),
      status: MemberStatus.ACTIVE,
      incarnation: 0
    };

    this.members.set(member.nodeId, member);
    this.membershipView.addMember(member);
    this.gossipProtocol.addNode(member.nodeId);
  }

  public requestLeave(nodeId: string): void {
    const member = this.members.get(nodeId);
    if (member) {
      member.status = MemberStatus.LEAVING;
      this.broadcastLeaveNotification(nodeId);
      
      setTimeout(() => {
        this.removeMember(nodeId);
      }, 5000);
    }
  }

  private removeMember(nodeId: string): void {
    this.members.delete(nodeId);
    this.membershipView.removeMember(nodeId);
    this.gossipProtocol.removeNode(nodeId);
  }

  private broadcastLeaveNotification(nodeId: string): void {
    // Notify all members about the leaving node
    this.gossipProtocol.broadcast({
      type: 'LEAVE',
      nodeId,
      timestamp: Date.now()
    });
  }

  private startHeartbeatMonitoring(): void {
    setInterval(() => {
      const now = Date.now();
      
      this.members.forEach(member => {
        const timeSinceLastHeartbeat = now - member.lastHeartbeat;
        
        if (member.status === MemberStatus.ACTIVE) {
          if (timeSinceLastHeartbeat > this.failureTimeout) {
            member.status = MemberStatus.FAILED;
            this.handleNodeFailure(member.nodeId);
          } else if (timeSinceLastHeartbeat > this.suspicionTimeout) {
            member.status = MemberStatus.SUSPECTED;
            this.gossipProtocol.broadcast({
              type: 'SUSPECT',
              nodeId: member.nodeId,
              timestamp: now
            });
          }
        }
      });
    }, this.heartbeatInterval);
  }

  private handleNodeFailure(nodeId: string): void {
    // Redistribute tasks from failed node
    this.gossipProtocol.broadcast({
      type: 'FAILURE',
      nodeId,
      timestamp: Date.now()
    });
    
    // Trigger task reassignment
    // In real implementation, reassign tasks from failed node
  }

  public updateHeartbeat(nodeId: string): void {
    const member = this.members.get(nodeId);
    if (member) {
      member.lastHeartbeat = Date.now();
      if (member.status === MemberStatus.SUSPECTED) {
        member.status = MemberStatus.ACTIVE;
        member.incarnation++;
      }
    }
  }

  public getActiveMembers(): Member[] {
    return Array.from(this.members.values())
      .filter(m => m.status === MemberStatus.ACTIVE);
  }

  public getMembershipView(): MembershipView {
    return this.membershipView;
  }

  private generateRequestId(): string {
    return `join-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}

interface Member {
  nodeId: string;
  address: string;
  publicKey: string;
  joinedAt: number;
  lastHeartbeat: number;
  status: MemberStatus;
  incarnation: number;
}

interface JoinRequest {
  id: string;
  node: Node;
  proof: JoinProof;
  timestamp: number;
  status: JoinStatus;
  votes: Map<string, boolean>;
}

interface JoinProof {
  signature: string;
  stake?: number;
  references?: string[];
  capabilities: NodeCapacity;
}

enum MemberStatus {
  ACTIVE = 'ACTIVE',
  SUSPECTED = 'SUSPECTED',
  FAILED = 'FAILED',
  LEAVING = 'LEAVING'
}

enum JoinStatus {
  PENDING = 'PENDING',
  APPROVED = 'APPROVED',
  REJECTED = 'REJECTED'
}

class GossipProtocol {
  private nodes: Set<string> = new Set();
  private messageHistory: Map<string, number> = new Map();
  private fanout: number = 3;

  addNode(nodeId: string): void {
    this.nodes.add(nodeId);
  }

  removeNode(nodeId: string): void {
    this.nodes.delete(nodeId);
  }

  broadcast(message: any): void {
    const messageId = this.generateMessageId();
    this.messageHistory.set(messageId, Date.now());
    
    // Select random nodes for gossip
    const targetNodes = this.selectRandomNodes(this.fanout);
    targetNodes.forEach(nodeId => {
      // In real implementation, send message to node
      console.log(`Gossiping to ${nodeId}:`, message);
    });
    
    // Clean old message history
    this.cleanMessageHistory();
  }

  private selectRandomNodes(count: number): string[] {
    const nodeArray = Array.from(this.nodes);
    const selected: string[] = [];
    
    for (let i = 0; i < Math.min(count, nodeArray.length); i++) {
      const index = Math.floor(Math.random() * nodeArray.length);
      selected.push(nodeArray[index]);
      nodeArray.splice(index, 1);
    }
    
    return selected;
  }

  private cleanMessageHistory(): void {
    const cutoff = Date.now() - 60000; // Keep messages for 1 minute
    
    this.messageHistory.forEach((timestamp, messageId) => {
      if (timestamp < cutoff) {
        this.messageHistory.delete(messageId);
      }
    });
  }

  private generateMessageId(): string {
    return `msg-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}

class MembershipView {
  private members: Map<string, Member> = new Map();
  private version: number = 0;

  addMember(member: Member): void {
    this.members.set(member.nodeId, member);
    this.version++;
  }

  removeMember(nodeId: string): void {
    this.members.delete(nodeId);
    this.version++;
  }

  getMembers(): Member[] {
    return Array.from(this.members.values());
  }

  getVersion(): number {
    return this.version;
  }

  merge(otherView: MembershipView): void {
    // Merge two membership views (used in gossip protocol)
    otherView.getMembers().forEach(member => {
      const existing = this.members.get(member.nodeId);
      if (!existing || member.incarnation > existing.incarnation) {
        this.members.set(member.nodeId, member);
      }
    });
    this.version++;
  }
}

// Main Decentralized Task Scheduler
class DecentralizedTaskScheduler {
  private nodeId: string;
  private consensus: RaftConsensus;
  private partitioner: JobPartitioner;
  private assignmentProtocol: WorkAssignmentProtocol;
  private loadBalancer: LoadBalancer;
  private validatorAssignment: ValidatorAssignment;
  private trustManager: TrustManager;
  private membershipManager: MembershipManager;
  private taskQueue: TaskQueue;
  private isRunning: boolean = false;

  constructor(nodeId: string) {
    this.nodeId = nodeId;
    this.consensus = new RaftConsensus(nodeId);
    this.partitioner = new JobPartitioner();
    this.assignmentProtocol = new WorkAssignmentProtocol();
    this.loadBalancer = new LoadBalancer();
    this.validatorAssignment = new ValidatorAssignment();
    this.trustManager = new TrustManager();
    this.membershipManager = new MembershipManager();
    this.taskQueue = new TaskQueue();
  }

  public start(): void {
    this.isRunning = true;
    console.log(`Decentralized Task Scheduler started on node ${this.nodeId}`);
    
    // Start scheduler loop
    this.schedulerLoop();
  }

  public stop(): void {
    this.isRunning = false;
    console.log(`Decentralized Task Scheduler stopped on node ${this.nodeId}`);
  }

  private async schedulerLoop(): Promise<void> {
    while (this.isRunning) {
      try {
        // Only leader performs scheduling
        if (this.consensus.isLeader()) {
          await this.performScheduling();
        }
        
        // Process local tasks
        await this.processLocalTasks();
        
        // Handle membership updates
        this.handleMembershipUpdates();
        
        await this.sleep(1000); // Schedule every second
      } catch (error) {
        console.error('Scheduler loop error:', error);
      }
    }
  }

  private async performScheduling(): Promise<void> {
    const pendingTasks = this.taskQueue.getPendingTasks();
    if (pendingTasks.length === 0) return;

    const activeNodes = this.membershipManager.getActiveMembers()
      .map(member => this.getNodeFromMember(member))
      .filter(node => !this.trustManager.isBlacklisted(node.id));

    // Partition tasks
    const partitions = this.partitioner.partitionTasks(pendingTasks, activeNodes, 'workload');

    // Assign tasks to nodes
    partitions.forEach((tasks, nodeId) => {
      const node = activeNodes.find(n => n.id === nodeId);
      if (node) {
        tasks.forEach(task => {
          const trustLevel = this.trustManager.evaluateNodeTrust(nodeId);
          const requiresValidation = this.requiresValidation(task, trustLevel);
          
          const assignment = this.assignmentProtocol.assignWork(task, node, requiresValidation);
          this.taskQueue.updateTaskStatus(task.id, TaskStatus.ASSIGNED);
          
          // If validation required, prepare validators
          if (requiresValidation && task.validationRequirement) {
            this.prepareValidation(task, assignment);
          }
        });
      }
    });
  }

  private requiresValidation(task: Task, trustLevel: TrustLevel): boolean {
    // High priority or sensitive tasks require validation
    if (task.priority > 8 || task.type === TaskType.VALIDATION) {
      return true;
    }
    
    // Low trust nodes require validation
    return trustLevel === TrustLevel.UNTRUSTED || trustLevel === TrustLevel.MINIMAL;
  }

  private prepareValidation(task: Task, assignment: Assignment): void {
    // Register validators for the task
    const validatorCount = task.validationRequirement?.validatorCount || 3;
    
    // Create validator entries
    for (let i = 0; i < validatorCount; i++) {
      const validator: Validator = {
        id: `validator-${i}-${task.id}`,
        publicKey: `pk-${i}`,
        reputation: 0.8,
        specializations: [task.type],
        status: ValidatorStatus.AVAILABLE,
        currentTask: null,
        successfulValidations: 0,
        failedValidations: 0,
        joinedAt: Date.now()
      };
      
      this.validatorAssignment.registerValidator(validator);
    }
  }

  private async processLocalTasks(): Promise<void> {
    // Process tasks assigned to this node
    // In real implementation, execute assigned tasks
  }

  private handleMembershipUpdates(): void {
    // Handle node failures and reassign tasks
    const failedNodes = this.membershipManager.getActiveMembers()
      .filter(m => m.status === MemberStatus.FAILED);
    
    failedNodes.forEach(member => {
      // Reassign tasks from failed node
      this.reassignTasksFromNode(member.nodeId);
    });
  }

  private reassignTasksFromNode(nodeId: string): void {
    // Find all tasks assigned to failed node and reassign them
    // In real implementation, query assignments and redistribute
    console.log(`Reassigning tasks from failed node: ${nodeId}`);
  }

  private getNodeFromMember(member: Member): Node {
    // Convert member to node
    return {
      id: member.nodeId,
      address: member.address,
      publicKey: member.publicKey,
      reputation: 0.5, // Default reputation
      capacity: {
        cpu: 100,
        memory: 1000,
        bandwidth: 100,
        currentLoad: 0
      },
      status: member.status === MemberStatus.ACTIVE ? NodeStatus.ACTIVE : NodeStatus.INACTIVE,
      lastHeartbeat: member.lastHeartbeat,
      joinedAt: member.joinedAt
    };
  }

  public submitTask(task: Task): void {
    this.taskQueue.addTask(task);
  }

  public getSchedulerState(): SchedulerState {
    return {
      nodeId: this.nodeId,
      isLeader: this.consensus.isLeader(),
      leaderId: this.consensus.getLeader(),
      activeNodes: this.membershipManager.getActiveMembers().length,
      pendingTasks: this.taskQueue.getPendingTasks().length,
      runningTasks: this.taskQueue.getRunningTasks().length,
      isRunning: this.isRunning
    };
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

interface SchedulerState {
  nodeId: string;
  isLeader: boolean;
  leaderId: string | null;
  activeNodes: number;
  pendingTasks: number;
  runningTasks: number;
  isRunning: boolean;
}

class TaskQueue {
  private tasks: Map<string, Task> = new Map();

  addTask(task: Task): void {
    this.tasks.set(task.id, task);
  }

  getPendingTasks(): Task[] {
    return Array.from(this.tasks.values())
      .filter(task => task.status === TaskStatus.PENDING)
      .sort((a, b) => b.priority - a.priority);
  }

  getRunningTasks(): Task[] {
    return Array.from(this.tasks.values())
      .filter(task => 
        task.status === TaskStatus.ASSIGNED || 
        task.status === TaskStatus.EXECUTING ||
        task.status === TaskStatus.VALIDATING
      );
  }

  updateTaskStatus(taskId: string, status: TaskStatus): void {
    const task = this.tasks.get(taskId);
    if (task) {
      task.status = status;
    }
  }

  removeTask(taskId: string): void {
    this.tasks.delete(taskId);
  }
}

// Export the main scheduler and related types
export {
  DecentralizedTaskScheduler,
  Node,
  Task,
  TaskType,
  TaskStatus,
  NodeStatus,
  TrustLevel,
  Assignment,
  AssignmentStatus,
  Validator,
  ValidationTask,
  Member,
  MemberStatus,
  SchedulerState
};

// Usage example
const scheduler = new DecentralizedTaskScheduler('node-1');
scheduler.start();

// Submit a task
const exampleTask: Task = {
  id: 'task-123',
  type: TaskType.COMPUTE,
  priority: 7,
  requirements: {
    minCpu: 50,
    minMemory: 512,
    estimatedDuration: 5000
  },
  payload: { data: 'example computation' },
  createdAt: Date.now(),
  status: TaskStatus.PENDING,
  validationRequirement: {
    validatorCount: 3,
    consensusThreshold: 0.67,
    validationTimeout: 10000
  }
};

scheduler.submitTask(exampleTask);