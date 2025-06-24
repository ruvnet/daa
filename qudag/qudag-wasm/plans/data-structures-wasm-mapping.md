# Data Structures WASM Mapping Strategy

## Executive Summary

This document provides a comprehensive mapping strategy for translating QuDAG's Rust data structures to WASM-compatible representations, focusing on memory efficiency, performance optimization, and JavaScript interoperability.

## 1. Memory Model Overview

### 1.1 WASM Memory Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    WASM Linear Memory                    │
├──────────────┬────────────────┬────────────────────────┤
│  Static Data │   Stack        │         Heap           │
│  (Globals)   │  (Function     │    (Dynamic Alloc)     │
│              │   Frames)      │                        │
├──────────────┴────────────────┴────────────────────────┤
│ 0x0000                                          0xFFFFFF │
└─────────────────────────────────────────────────────────┘
```

### 1.2 Memory Layout Principles
- **Alignment**: 8-byte alignment for 64-bit values
- **Packing**: Minimize padding between fields
- **Locality**: Group frequently accessed data
- **Growth**: Pre-allocate for expected growth patterns

## 2. Core Data Structure Mappings

### 2.1 VertexId Mapping

**Rust Structure**:
```rust
pub struct VertexId(Vec<u8>);
```

**WASM Representation**:
```
Memory Layout:
┌─────────┬─────────┬──────────────┐
│ Length  │ Capacity│ Data...      │
│ (u32)   │ (u32)   │ (bytes)      │
└─────────┴─────────┴──────────────┘

JavaScript Interface:
class VertexId {
    private ptr: number;     // WASM pointer
    private len: number;     // Data length
    
    static fromBytes(bytes: Uint8Array): VertexId
    toBytes(): Uint8Array
    equals(other: VertexId): boolean
    hash(): number
}
```

**Optimization Strategy**:
- Use fixed 32-byte IDs to avoid dynamic allocation
- Implement object pooling for frequent allocations
- Cache hash values to avoid recomputation

### 2.2 Vertex Structure Mapping

**Rust Structure**:
```rust
pub struct Vertex {
    pub id: VertexId,
    pub parents: Vec<VertexId>,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}
```

**WASM Memory Layout**:
```
Vertex Layout (Optimized):
┌────────────┬────────────┬─────────────┬───────────┬─────────────┐
│ Flags      │ ID Offset  │ Parents Ptr │ Timestamp │ Payload Ptr │
│ (u32)      │ (u32)      │ (u32)       │ (u64)     │ (u32)       │
├────────────┼────────────┼─────────────┼───────────┼─────────────┤
│ Sig Ptr    │ Parent Cnt │ Reserved    │           │             │
│ (u32)      │ (u32)      │ (u32)       │           │             │
└────────────┴────────────┴─────────────┴───────────┴─────────────┘

Flags Bitmap:
Bit 0: Has signature
Bit 1: Is finalized
Bit 2: Is orphan
Bit 3-31: Reserved
```

**JavaScript Wrapper**:
```typescript
interface VertexView {
    readonly id: VertexId;
    readonly parents: ReadonlyArray<VertexId>;
    readonly payload: Uint8Array;
    readonly timestamp: bigint;
    readonly signature?: Uint8Array;
    
    // Methods
    validate(): boolean;
    hash(): Uint8Array;
    serialize(): Uint8Array;
}

class WASMVertex implements VertexView {
    constructor(private memory: WebAssembly.Memory, 
                private offset: number) {}
    
    get id(): VertexId {
        const idOffset = this.memory.getUint32(this.offset + 4, true);
        return VertexId.fromPointer(this.memory, idOffset);
    }
    
    // Additional getters with lazy loading
}
```

### 2.3 DAG Storage Mapping

**Rust Structure**:
```rust
pub struct Dag {
    pub vertices: Arc<RwLock<HashMap<VertexId, Vertex>>>,
    // ... other fields
}
```

**WASM Optimized Structure**:
```
DAG Storage Layout:
┌─────────────────────────────────────────────────┐
│              DAG Header (64 bytes)               │
├─────────────┬───────────┬───────────┬──────────┤
│ Version     │ Vertex Cnt│ Edge Count│ Tip Count│
│ (u32)       │ (u32)     │ (u32)     │ (u32)    │
├─────────────┴───────────┴───────────┴──────────┤
│              Hash Table Buckets                  │
├─────────────────────────────────────────────────┤
│              Vertex Data Pool                    │
├─────────────────────────────────────────────────┤
│              Edge Index                          │
└─────────────────────────────────────────────────┘

Hash Table Entry:
┌──────────┬──────────┬──────────┬──────────┐
│ Key Hash │ Next Ptr │ Vertex   │ Metadata │
│ (u32)    │ (u32)    │ Offset   │ (u32)    │
└──────────┴──────────┴──────────┴──────────┘
```

**Memory Pool Management**:
```typescript
class MemoryPool {
    private freeList: number[] = [];
    private nextOffset: number;
    private readonly chunkSize: number;
    
    allocate(size: number): number {
        // First-fit allocation strategy
        const aligned = (size + 7) & ~7; // 8-byte align
        
        // Check free list
        for (let i = 0; i < this.freeList.length; i++) {
            const chunk = this.freeList[i];
            if (this.getSize(chunk) >= aligned) {
                return this.splitChunk(chunk, aligned);
            }
        }
        
        // Allocate new chunk
        return this.growHeap(aligned);
    }
    
    deallocate(offset: number, size: number): void {
        // Coalesce with adjacent free chunks
        this.coalesce(offset, size);
    }
}
```

### 2.4 Graph Traversal Index

**Optimized Structure for WASM**:
```
Traversal Index:
┌─────────────────────────────────────────────┐
│          Topological Order Array             │
├─────────┬─────────┬─────────┬──────────────┤
│ Level 0 │ Level 1 │ Level 2 │ ...          │
└─────────┴─────────┴─────────┴──────────────┘

Level Structure:
┌──────────┬──────────┬────────────────────┐
│ Count    │ Start    │ Vertex IDs...      │
│ (u32)    │ Offset   │ (u32[])            │
└──────────┴──────────┴────────────────────┘

Parent-Child Index (Adjacency List):
┌──────────┬────────────────────────────────┐
│ Vertex   │ Children List                  │
│ Offset   │ [Count, Child1, Child2, ...]   │
└──────────┴────────────────────────────────┘
```

**Traversal Algorithm Optimization**:
```typescript
class TraversalIndex {
    private levels: Uint32Array[];
    private parentIndex: Map<number, Uint32Array>;
    private childIndex: Map<number, Uint32Array>;
    
    // Breadth-first traversal with level tracking
    traverse(startVertex: number, visitor: (v: number, level: number) => void) {
        const queue = new Uint32Array(this.vertexCount);
        let queueStart = 0, queueEnd = 0;
        
        queue[queueEnd++] = startVertex;
        const visited = new Uint8Array(this.vertexCount);
        
        while (queueStart < queueEnd) {
            const vertex = queue[queueStart++];
            if (visited[vertex]) continue;
            
            visited[vertex] = 1;
            const level = this.getLevel(vertex);
            visitor(vertex, level);
            
            // Add children to queue
            const children = this.childIndex.get(vertex);
            if (children) {
                for (let i = 0; i < children.length; i++) {
                    queue[queueEnd++] = children[i];
                }
            }
        }
    }
}
```

### 2.5 Consensus State Mapping

**Rust Structure**:
```rust
pub struct Confidence {
    pub value: f64,
    pub positive_votes: usize,
    pub negative_votes: usize,
    pub last_updated: Instant,
}
```

**WASM Compact Representation**:
```
Consensus State Entry (16 bytes):
┌────────────┬────────────┬────────────┬────────────┐
│ Confidence │ Pos Votes  │ Neg Votes  │ Timestamp  │
│ (f32)      │ (u16)      │ (u16)      │ (u64)      │
└────────────┴────────────┴────────────┴────────────┘

Packed Status Byte:
Bits 0-1: Status (Pending=0, Accepted=1, Rejected=2, Final=3)
Bits 2-3: Priority level
Bits 4-7: Flags/Reserved
```

### 2.6 Message Queue Mapping

**Efficient Ring Buffer Implementation**:
```
Ring Buffer Layout:
┌─────────────────────────────────────────────┐
│ Header: [Head, Tail, Capacity, Count]       │
├─────────────────────────────────────────────┤
│ Message Slot 0                              │
│ Message Slot 1                              │
│ ...                                         │
│ Message Slot N-1                            │
└─────────────────────────────────────────────┘

Message Slot:
┌──────────┬──────────┬──────────┬───────────┐
│ Status   │ Priority │ Size     │ Data Ptr  │
│ (u8)     │ (u8)     │ (u16)    │ (u32)     │
└──────────┴──────────┴──────────┴───────────┘
```

## 3. Serialization Strategy

### 3.1 Binary Format Specification

```
QuDAG Binary Format (QBF):
┌────────────┬────────────┬─────────────┬──────────┐
│ Magic      │ Version    │ Flags       │ Checksum │
│ "QDAG"     │ (u16)      │ (u16)       │ (u32)    │
├────────────┴────────────┴─────────────┴──────────┤
│                  Vertex Count (u32)               │
├───────────────────────────────────────────────────┤
│                  Vertex Data...                   │
├───────────────────────────────────────────────────┤
│                  Edge Count (u32)                 │
├───────────────────────────────────────────────────┤
│                  Edge Data...                     │
└───────────────────────────────────────────────────┘
```

### 3.2 Compression Strategy

```typescript
class CompressionStrategy {
    // Variable-length integer encoding
    static encodeVarInt(value: number): Uint8Array {
        const bytes = [];
        while (value > 0x7F) {
            bytes.push((value & 0x7F) | 0x80);
            value >>>= 7;
        }
        bytes.push(value);
        return new Uint8Array(bytes);
    }
    
    // Delta encoding for timestamps
    static deltaEncode(timestamps: BigUint64Array): Uint8Array {
        const deltas = new BigUint64Array(timestamps.length);
        deltas[0] = timestamps[0];
        for (let i = 1; i < timestamps.length; i++) {
            deltas[i] = timestamps[i] - timestamps[i-1];
        }
        return this.compressDeltas(deltas);
    }
}
```

## 4. Memory Management Strategies

### 4.1 Object Pooling

```typescript
class ObjectPool<T> {
    private pool: T[] = [];
    private factory: () => T;
    private reset: (obj: T) => void;
    private maxSize: number;
    
    acquire(): T {
        if (this.pool.length > 0) {
            return this.pool.pop()!;
        }
        return this.factory();
    }
    
    release(obj: T): void {
        if (this.pool.length < this.maxSize) {
            this.reset(obj);
            this.pool.push(obj);
        }
    }
}

// Vertex pool example
const vertexPool = new ObjectPool<WASMVertex>({
    factory: () => new WASMVertex(),
    reset: (v) => v.clear(),
    maxSize: 1000
});
```

### 4.2 Memory Pressure Management

```typescript
class MemoryManager {
    private readonly highWaterMark = 0.8; // 80% of available memory
    private readonly lowWaterMark = 0.6;  // 60% of available memory
    
    async checkPressure(): Promise<void> {
        const usage = this.getMemoryUsage();
        const ratio = usage / this.getMemoryLimit();
        
        if (ratio > this.highWaterMark) {
            await this.performGC();
        }
    }
    
    private async performGC(): Promise<void> {
        // 1. Evict least recently used vertices
        // 2. Compress inactive data
        // 3. Move to IndexedDB if necessary
    }
}
```

## 5. Persistence Layer

### 5.1 IndexedDB Schema

```typescript
interface DAGDatabase {
    vertices: {
        key: Uint8Array;         // VertexId
        value: {
            data: Uint8Array;    // Serialized vertex
            timestamp: number;    // Last access
            level: number;       // Topological level
        };
        indexes: {
            byTimestamp: number;
            byLevel: number;
        };
    };
    
    edges: {
        key: [Uint8Array, Uint8Array]; // [from, to]
        value: {
            weight: number;
            metadata: Uint8Array;
        };
    };
    
    metadata: {
        key: string;
        value: any;
    };
}
```

### 5.2 Caching Strategy

```typescript
class HierarchicalCache {
    private l1Cache: LRUCache<VertexId, Vertex>;    // Hot data
    private l2Cache: WeakMap<VertexId, Vertex>;     // Warm data
    private l3Storage: IndexedDBStorage;            // Cold data
    
    async get(id: VertexId): Promise<Vertex | null> {
        // Check L1 (fastest)
        let vertex = this.l1Cache.get(id);
        if (vertex) return vertex;
        
        // Check L2
        vertex = this.l2Cache.get(id);
        if (vertex) {
            this.l1Cache.set(id, vertex);
            return vertex;
        }
        
        // Load from L3 (slowest)
        vertex = await this.l3Storage.load(id);
        if (vertex) {
            this.promote(id, vertex);
        }
        return vertex;
    }
}
```

## 6. Performance Optimization Techniques

### 6.1 SIMD Operations (when available)

```typescript
class SIMDOperations {
    static compareVertexIds(a: Uint8Array, b: Uint8Array): boolean {
        if (!WebAssembly.validate || !SIMD) {
            return this.fallbackCompare(a, b);
        }
        
        // WASM SIMD comparison
        const chunks = Math.floor(a.length / 16);
        for (let i = 0; i < chunks; i++) {
            const va = SIMD.v128.load(a, i * 16);
            const vb = SIMD.v128.load(b, i * 16);
            const eq = SIMD.i8x16.eq(va, vb);
            if (!SIMD.v128.all_true(eq)) {
                return false;
            }
        }
        
        // Handle remaining bytes
        return this.fallbackCompare(
            a.subarray(chunks * 16),
            b.subarray(chunks * 16)
        );
    }
}
```

### 6.2 Batch Operations

```typescript
interface BatchOperation {
    type: 'insert' | 'update' | 'delete';
    vertex: Vertex;
}

class BatchProcessor {
    private queue: BatchOperation[] = [];
    private processing = false;
    
    async processBatch(): Promise<void> {
        if (this.processing || this.queue.length === 0) return;
        
        this.processing = true;
        const batch = this.queue.splice(0, 1000); // Process up to 1000
        
        // Sort by operation type for efficiency
        batch.sort((a, b) => a.type.localeCompare(b.type));
        
        // Process in WASM
        await this.wasmModule.processBatch(batch);
        this.processing = false;
        
        // Process next batch if any
        if (this.queue.length > 0) {
            setTimeout(() => this.processBatch(), 0);
        }
    }
}
```

## 7. Debugging and Profiling Support

### 7.1 Memory Profiling

```typescript
class MemoryProfiler {
    private allocations: Map<number, AllocationInfo> = new Map();
    
    trackAllocation(ptr: number, size: number, type: string): void {
        this.allocations.set(ptr, {
            size,
            type,
            timestamp: performance.now(),
            stackTrace: new Error().stack
        });
    }
    
    generateReport(): MemoryReport {
        const byType = new Map<string, number>();
        let totalSize = 0;
        
        for (const [ptr, info] of this.allocations) {
            totalSize += info.size;
            byType.set(info.type, (byType.get(info.type) || 0) + info.size);
        }
        
        return {
            totalAllocated: totalSize,
            allocationsByType: byType,
            largestAllocations: this.getLargestAllocations(10)
        };
    }
}
```

### 7.2 Performance Monitoring

```typescript
class PerformanceMonitor {
    private metrics: Map<string, PerformanceMetric> = new Map();
    
    measure<T>(name: string, fn: () => T): T {
        const start = performance.now();
        try {
            const result = fn();
            this.recordSuccess(name, performance.now() - start);
            return result;
        } catch (error) {
            this.recordFailure(name, performance.now() - start);
            throw error;
        }
    }
    
    getMetrics(name: string): PerformanceMetric {
        return this.metrics.get(name) || {
            count: 0,
            totalTime: 0,
            avgTime: 0,
            minTime: Infinity,
            maxTime: 0
        };
    }
}
```

## 8. Migration Guidelines

### 8.1 Incremental Migration Path
1. **Phase 1**: Core data structures (VertexId, Vertex)
2. **Phase 2**: Storage layer (HashMap → WASM-optimized storage)
3. **Phase 3**: Traversal algorithms
4. **Phase 4**: Consensus state management
5. **Phase 5**: Full system integration

### 8.2 Compatibility Layer

```typescript
// Bridge between Rust and WASM representations
class CompatibilityBridge {
    static fromRustVertex(rustBytes: Uint8Array): WASMVertex {
        // Deserialize Rust format
        const decoded = decode_vertex_from_rust(rustBytes);
        
        // Create WASM representation
        const wasmVertex = new WASMVertex();
        wasmVertex.id = decoded.id;
        wasmVertex.parents = decoded.parents;
        // ... map other fields
        
        return wasmVertex;
    }
    
    static toRustVertex(wasmVertex: WASMVertex): Uint8Array {
        // Serialize to Rust-compatible format
        return encode_vertex_for_rust(wasmVertex);
    }
}
```

## 9. Benchmarking Framework

```typescript
interface BenchmarkResult {
    operation: string;
    opsPerSecond: number;
    avgLatency: number;
    p95Latency: number;
    p99Latency: number;
}

class Benchmark {
    static async runDAGBenchmarks(): Promise<BenchmarkResult[]> {
        const results: BenchmarkResult[] = [];
        
        // Vertex creation benchmark
        results.push(await this.benchmarkVertexCreation());
        
        // Traversal benchmark
        results.push(await this.benchmarkTraversal());
        
        // Consensus update benchmark
        results.push(await this.benchmarkConsensus());
        
        return results;
    }
}
```

## 10. Recommendations and Best Practices

### 10.1 Memory Efficiency
1. Use fixed-size allocations where possible
2. Implement aggressive object pooling
3. Compress data before storage
4. Use typed arrays for binary data
5. Minimize cross-boundary data copies

### 10.2 Performance Optimization
1. Batch operations to reduce overhead
2. Use Web Workers for parallel processing
3. Implement lazy loading for large graphs
4. Cache computed values aggressively
5. Profile and optimize hot paths

### 10.3 Maintainability
1. Keep clear separation between WASM and JS
2. Document memory layout thoroughly
3. Provide debugging tools and visualizers
4. Implement comprehensive error handling
5. Version data formats for compatibility

## Conclusion

This mapping strategy provides a solid foundation for implementing QuDAG data structures in WASM. The focus on memory efficiency, performance optimization, and JavaScript interoperability ensures that the WASM implementation will be both performant and maintainable. The hierarchical caching system and persistence layer enable handling of large-scale DAGs while respecting browser memory constraints.