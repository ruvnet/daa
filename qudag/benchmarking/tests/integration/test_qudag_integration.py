"""Integration tests for QuDAG benchmarking interactions."""

import asyncio
import json
import time
from pathlib import Path
from unittest.mock import Mock, AsyncMock, patch

import pytest

from benchmarking.src.qudag_interface import (
    QuDAGBenchmarkClient,
    QuDAGTransaction,
    QuDAGMetrics,
    DAGQueryBuilder,
    QuDAGConnectionPool
)
from benchmarking.src.benchmark_scenarios import (
    TransactionThroughputScenario,
    ConsensusLatencyScenario,
    NetworkResilienceScenario,
    ScalabilityTestScenario
)


class TestQuDAGBenchmarkClient:
    """Test QuDAG client for benchmarking."""
    
    @pytest.mark.asyncio
    async def test_client_initialization(self):
        """Test client initialization and configuration."""
        config = {
            "node_url": "localhost:8080",
            "timeout": 30,
            "max_retries": 3,
            "connection_pool_size": 10
        }
        
        client = QuDAGBenchmarkClient(config)
        
        assert client.node_url == "localhost:8080"
        assert client.timeout == 30
        assert client.max_retries == 3
        assert client.connection_pool is not None
        
    @pytest.mark.asyncio
    async def test_client_connection(self, mock_qudag):
        """Test establishing connection to QuDAG node."""
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        result = await client.connect()
        
        assert result is True
        mock_qudag.connect.assert_called_once_with("test:8080")
        
    @pytest.mark.asyncio
    async def test_client_connection_retry(self):
        """Test connection retry on failure."""
        client = QuDAGBenchmarkClient({
            "node_url": "test:8080",
            "max_retries": 3
        })
        
        # Mock transport to fail twice then succeed
        mock_transport = AsyncMock()
        mock_transport.connect.side_effect = [
            ConnectionError("Failed"),
            ConnectionError("Failed again"),
            True
        ]
        client._transport = mock_transport
        
        result = await client.connect()
        
        assert result is True
        assert mock_transport.connect.call_count == 3
        
    @pytest.mark.asyncio
    async def test_submit_single_transaction(self, mock_qudag):
        """Test submitting a single transaction."""
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        transaction = QuDAGTransaction(
            data=b"test transaction data",
            metadata={"type": "benchmark", "size": 21}
        )
        
        tx_hash = await client.submit_transaction(transaction)
        
        assert tx_hash == "tx_hash_12345"
        mock_qudag.submit_transaction.assert_called_once()
        
    @pytest.mark.asyncio
    async def test_submit_batch_transactions(self, mock_qudag):
        """Test submitting batch of transactions."""
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        # Configure mock to return different hashes
        mock_qudag.submit_transaction.side_effect = [
            f"tx_hash_{i}" for i in range(10)
        ]
        
        transactions = [
            QuDAGTransaction(
                data=f"transaction_{i}".encode(),
                metadata={"index": i}
            )
            for i in range(10)
        ]
        
        results = await client.submit_batch(transactions)
        
        assert len(results) == 10
        assert all(result["status"] == "success" for result in results)
        assert results[0]["tx_hash"] == "tx_hash_0"
        assert results[9]["tx_hash"] == "tx_hash_9"
        
    @pytest.mark.asyncio
    async def test_parallel_transaction_submission(self, mock_qudag):
        """Test parallel transaction submission."""
        client = QuDAGBenchmarkClient({
            "node_url": "test:8080",
            "parallel_submissions": 5
        })
        client._transport = mock_qudag
        
        # Track submission order
        submission_times = []
        
        async def track_submission(data):
            submission_times.append(time.time())
            await asyncio.sleep(0.01)  # Simulate processing
            return f"tx_hash_{len(submission_times)}"
            
        mock_qudag.submit_transaction.side_effect = track_submission
        
        transactions = [
            QuDAGTransaction(data=f"tx_{i}".encode())
            for i in range(20)
        ]
        
        results = await client.submit_parallel(transactions, workers=5)
        
        assert len(results) == 20
        assert all(r["status"] == "success" for r in results)
        
        # Verify parallel execution
        # Check that multiple transactions were submitted close together
        time_diffs = [
            submission_times[i+1] - submission_times[i]
            for i in range(len(submission_times)-1)
        ]
        # At least some submissions should be very close in time
        assert any(diff < 0.005 for diff in time_diffs)
        
    @pytest.mark.asyncio
    async def test_query_dag_state(self, mock_qudag):
        """Test querying DAG state."""
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        query = DAGQueryBuilder()\
            .with_height_range(0, 10)\
            .with_vertex_limit(100)\
            .build()
            
        results = await client.query_dag(query)
        
        assert len(results) == 2
        assert results[0]["id"] == "vertex1"
        assert results[1]["id"] == "vertex2"
        mock_qudag.query_dag.assert_called_once_with(query)
        
    @pytest.mark.asyncio
    async def test_get_performance_metrics(self, mock_qudag):
        """Test retrieving performance metrics."""
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        metrics = await client.get_metrics()
        
        assert isinstance(metrics, QuDAGMetrics)
        assert metrics.transactions_per_second == 1000
        assert metrics.latency_ms == 50
        assert metrics.memory_mb == 256
        assert metrics.cpu_percent == 45.5
        
    @pytest.mark.asyncio
    async def test_monitor_metrics_stream(self, mock_qudag):
        """Test continuous metrics monitoring."""
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        # Mock changing metrics
        metric_values = [
            {"transactions_per_second": 1000 + i*100, "latency_ms": 50 - i*5}
            for i in range(5)
        ]
        mock_qudag.get_metrics.side_effect = metric_values
        
        collected_metrics = []
        
        async def collect_metrics():
            async for metrics in client.monitor_metrics(interval=0.01, duration=0.05):
                collected_metrics.append(metrics)
                
        await collect_metrics()
        
        assert len(collected_metrics) >= 4
        assert collected_metrics[0].transactions_per_second == 1000
        assert collected_metrics[-1].transactions_per_second >= 1300


class TestQuDAGConnectionPool:
    """Test connection pooling for QuDAG."""
    
    @pytest.mark.asyncio
    async def test_pool_initialization(self):
        """Test connection pool initialization."""
        pool = QuDAGConnectionPool(
            size=5,
            node_url="localhost:8080",
            timeout=30
        )
        
        await pool.initialize()
        
        assert pool.size == 5
        assert len(pool._connections) == 5
        assert pool._available.qsize() == 5
        
    @pytest.mark.asyncio
    async def test_acquire_and_release_connection(self):
        """Test acquiring and releasing connections."""
        pool = QuDAGConnectionPool(size=2, node_url="test:8080")
        await pool.initialize()
        
        # Acquire connection
        conn1 = await pool.acquire()
        assert conn1 is not None
        assert pool._available.qsize() == 1
        
        # Acquire second connection
        conn2 = await pool.acquire()
        assert conn2 is not None
        assert pool._available.qsize() == 0
        assert conn1 != conn2
        
        # Release connections
        await pool.release(conn1)
        assert pool._available.qsize() == 1
        
        await pool.release(conn2)
        assert pool._available.qsize() == 2
        
    @pytest.mark.asyncio
    async def test_pool_exhaustion_waiting(self):
        """Test waiting when pool is exhausted."""
        pool = QuDAGConnectionPool(size=1, node_url="test:8080")
        await pool.initialize()
        
        # Acquire the only connection
        conn1 = await pool.acquire()
        
        # Try to acquire another (should wait)
        acquire_task = asyncio.create_task(pool.acquire())
        
        # Should not complete immediately
        await asyncio.sleep(0.01)
        assert not acquire_task.done()
        
        # Release connection
        await pool.release(conn1)
        
        # Now acquire should complete
        conn2 = await acquire_task
        assert conn2 == conn1  # Same connection reused
        
    @pytest.mark.asyncio
    async def test_pool_connection_health_check(self):
        """Test connection health checking."""
        pool = QuDAGConnectionPool(
            size=3,
            node_url="test:8080",
            health_check_interval=0.05
        )
        
        # Mock connections
        mock_connections = [AsyncMock() for _ in range(3)]
        for i, conn in enumerate(mock_connections):
            conn.is_healthy = AsyncMock(return_value=i != 1)  # Second connection unhealthy
            
        pool._connections = mock_connections
        await pool.initialize()
        
        # Run health check
        await pool._health_check()
        
        # Verify health check was called
        for conn in mock_connections:
            conn.is_healthy.assert_called_once()
            
        # Unhealthy connection should be replaced
        assert mock_connections[1] not in pool._connections


class TestTransactionThroughputScenario:
    """Test transaction throughput benchmarking scenario."""
    
    @pytest.mark.asyncio
    async def test_scenario_initialization(self):
        """Test throughput scenario initialization."""
        config = {
            "target_tps": 1000,
            "duration_seconds": 60,
            "transaction_size": 1000,
            "batch_size": 100
        }
        
        scenario = TransactionThroughputScenario(config)
        
        assert scenario.target_tps == 1000
        assert scenario.duration_seconds == 60
        assert scenario.transaction_size == 1000
        assert scenario.batch_size == 100
        
    @pytest.mark.asyncio
    async def test_generate_workload(self):
        """Test workload generation for throughput testing."""
        scenario = TransactionThroughputScenario({
            "target_tps": 100,
            "duration_seconds": 1,
            "transaction_size": 500
        })
        
        workload = scenario.generate_workload()
        
        assert workload.total_transactions == 100
        assert workload.transaction_size == 500
        assert len(workload.transactions) == 100
        assert all(len(tx.data) == 500 for tx in workload.transactions)
        
    @pytest.mark.asyncio
    async def test_execute_throughput_test(self, mock_qudag):
        """Test executing throughput benchmark."""
        scenario = TransactionThroughputScenario({
            "target_tps": 10,
            "duration_seconds": 0.1,
            "transaction_size": 100
        })
        
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        # Run scenario
        results = await scenario.execute(client)
        
        assert results.scenario_name == "TransactionThroughput"
        assert results.total_transactions >= 1
        assert results.duration > 0
        assert results.actual_tps > 0
        assert "latency_percentiles" in results.metrics
        
    @pytest.mark.asyncio
    async def test_rate_limiting(self, mock_qudag):
        """Test transaction rate limiting."""
        scenario = TransactionThroughputScenario({
            "target_tps": 50,
            "duration_seconds": 0.2,
            "enable_rate_limiting": True
        })
        
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        # Track submission times
        submission_times = []
        
        async def track_submission(data):
            submission_times.append(time.time())
            return f"tx_{len(submission_times)}"
            
        mock_qudag.submit_transaction.side_effect = track_submission
        
        await scenario.execute(client)
        
        # Calculate actual TPS
        if len(submission_times) > 1:
            duration = submission_times[-1] - submission_times[0]
            actual_tps = len(submission_times) / duration
            
            # Should be close to target TPS (within 20%)
            assert abs(actual_tps - 50) / 50 < 0.2


class TestConsensusLatencyScenario:
    """Test consensus latency benchmarking scenario."""
    
    @pytest.mark.asyncio
    async def test_scenario_initialization(self):
        """Test consensus latency scenario initialization."""
        config = {
            "test_rounds": 100,
            "confirmation_threshold": 0.67,
            "timeout_seconds": 30
        }
        
        scenario = ConsensusLatencyScenario(config)
        
        assert scenario.test_rounds == 100
        assert scenario.confirmation_threshold == 0.67
        assert scenario.timeout_seconds == 30
        
    @pytest.mark.asyncio
    async def test_measure_consensus_latency(self, mock_qudag):
        """Test measuring consensus achievement latency."""
        scenario = ConsensusLatencyScenario({
            "test_rounds": 5,
            "confirmation_threshold": 0.5
        })
        
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        # Mock consensus confirmation
        async def mock_check_consensus(tx_hash):
            await asyncio.sleep(0.01)  # Simulate latency
            return {
                "confirmed": True,
                "confirmation_level": 0.75,
                "confirming_nodes": 6,
                "total_nodes": 8
            }
            
        client.check_consensus = AsyncMock(side_effect=mock_check_consensus)
        
        results = await scenario.execute(client)
        
        assert results.scenario_name == "ConsensusLatency"
        assert results.total_measurements == 5
        assert results.mean_latency_ms > 0
        assert results.min_latency_ms > 0
        assert results.max_latency_ms >= results.min_latency_ms
        
    @pytest.mark.asyncio
    async def test_consensus_timeout_handling(self, mock_qudag):
        """Test handling of consensus timeout."""
        scenario = ConsensusLatencyScenario({
            "test_rounds": 3,
            "timeout_seconds": 0.05
        })
        
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        # Mock slow consensus
        async def mock_slow_consensus(tx_hash):
            await asyncio.sleep(0.1)  # Longer than timeout
            return {"confirmed": False}
            
        client.check_consensus = AsyncMock(side_effect=mock_slow_consensus)
        
        results = await scenario.execute(client)
        
        assert results.timeout_count > 0
        assert results.success_rate < 1.0


class TestNetworkResilienceScenario:
    """Test network resilience benchmarking scenario."""
    
    @pytest.mark.asyncio
    async def test_scenario_initialization(self):
        """Test network resilience scenario initialization."""
        config = {
            "failure_scenarios": ["node_failure", "network_partition"],
            "failure_rate": 0.1,
            "recovery_timeout": 30,
            "test_duration": 300
        }
        
        scenario = NetworkResilienceScenario(config)
        
        assert scenario.failure_scenarios == ["node_failure", "network_partition"]
        assert scenario.failure_rate == 0.1
        assert scenario.recovery_timeout == 30
        
    @pytest.mark.asyncio
    async def test_simulate_node_failures(self, mock_qudag):
        """Test simulating node failures."""
        scenario = NetworkResilienceScenario({
            "failure_scenarios": ["node_failure"],
            "failure_rate": 0.5,
            "test_duration": 1
        })
        
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        # Mock node operations
        client.get_node_count = AsyncMock(return_value=10)
        client.simulate_node_failure = AsyncMock()
        client.restore_node = AsyncMock()
        
        results = await scenario.execute(client)
        
        assert results.scenario_name == "NetworkResilience"
        assert results.total_failures > 0
        assert client.simulate_node_failure.called
        
    @pytest.mark.asyncio
    async def test_measure_recovery_time(self, mock_qudag):
        """Test measuring system recovery time."""
        scenario = NetworkResilienceScenario({
            "failure_scenarios": ["network_partition"],
            "measure_recovery": True
        })
        
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        # Mock partition and recovery
        recovery_times = []
        
        async def mock_create_partition():
            recovery_times.append(time.time())
            return True
            
        async def mock_check_recovery():
            if len(recovery_times) > 0:
                elapsed = time.time() - recovery_times[-1]
                return elapsed > 0.05  # 50ms recovery time
            return False
            
        client.create_network_partition = AsyncMock(side_effect=mock_create_partition)
        client.is_fully_connected = AsyncMock(side_effect=mock_check_recovery)
        
        results = await scenario.execute(client)
        
        assert results.mean_recovery_time_ms > 40
        assert results.recovery_success_rate > 0


class TestScalabilityTestScenario:
    """Test scalability benchmarking scenario."""
    
    @pytest.mark.asyncio
    async def test_scenario_initialization(self):
        """Test scalability scenario initialization."""
        config = {
            "initial_nodes": 5,
            "max_nodes": 100,
            "scale_step": 10,
            "workload_per_node": 100,
            "measure_metrics": ["tps", "latency", "memory"]
        }
        
        scenario = ScalabilityTestScenario(config)
        
        assert scenario.initial_nodes == 5
        assert scenario.max_nodes == 100
        assert scenario.scale_step == 10
        assert "tps" in scenario.measure_metrics
        
    @pytest.mark.asyncio
    async def test_scale_up_testing(self, mock_qudag):
        """Test scale-up performance testing."""
        scenario = ScalabilityTestScenario({
            "initial_nodes": 2,
            "max_nodes": 6,
            "scale_step": 2,
            "workload_per_node": 10
        })
        
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        # Mock scaling operations
        current_nodes = {"count": 2}
        
        async def mock_add_nodes(count):
            current_nodes["count"] += count
            return True
            
        async def mock_get_metrics():
            # Simulate decreasing performance with scale
            base_tps = 1000
            degradation = (current_nodes["count"] - 2) * 50
            return {
                "transactions_per_second": base_tps - degradation,
                "latency_ms": 50 + degradation * 0.5,
                "node_count": current_nodes["count"]
            }
            
        client.add_nodes = AsyncMock(side_effect=mock_add_nodes)
        mock_qudag.get_metrics.side_effect = mock_get_metrics
        
        results = await scenario.execute(client)
        
        assert results.scenario_name == "ScalabilityTest"
        assert len(results.scale_points) == 3  # 2, 4, 6 nodes
        assert results.scale_points[0]["node_count"] == 2
        assert results.scale_points[-1]["node_count"] == 6
        
        # Verify performance degradation captured
        assert results.scale_points[0]["tps"] > results.scale_points[-1]["tps"]
        assert results.scale_points[0]["latency_ms"] < results.scale_points[-1]["latency_ms"]
        
    @pytest.mark.asyncio
    async def test_identify_scalability_limits(self, mock_qudag):
        """Test identifying system scalability limits."""
        scenario = ScalabilityTestScenario({
            "initial_nodes": 5,
            "max_nodes": 50,
            "scale_step": 5,
            "performance_threshold": {
                "min_tps": 500,
                "max_latency_ms": 100
            }
        })
        
        client = QuDAGBenchmarkClient({"node_url": "test:8080"})
        client._transport = mock_qudag
        
        # Mock performance degradation
        async def mock_metrics_degrading(node_count):
            tps = 1000 - (node_count * 20)  # Degrades with scale
            latency = 50 + (node_count * 2)   # Increases with scale
            
            return {
                "transactions_per_second": max(0, tps),
                "latency_ms": latency,
                "node_count": node_count
            }
            
        client.get_current_node_count = AsyncMock(side_effect=lambda: 5)
        
        results = await scenario.execute(client)
        
        # Should identify the scale limit
        assert results.scalability_limit is not None
        assert results.scalability_limit["reason"] in ["tps_threshold", "latency_threshold"]
        assert results.scalability_limit["node_count"] < 50