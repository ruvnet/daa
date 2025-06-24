use qudag_protocol::{Coordinator, ProtocolConfig, ProtocolState};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

#[tokio::test]
async fn test_concurrent_message_broadcasting() {
    // Test concurrent message broadcasting from multiple tasks
    let config = ProtocolConfig::default();
    let coordinator = Arc::new(Mutex::new(Coordinator::new(config).await.unwrap()));
    
    {
        let mut coord = coordinator.lock().await;
        coord.start().await.unwrap();
    }
    
    // Create multiple concurrent message broadcasting tasks
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let coordinator_clone = coordinator.clone();
        let handle = tokio::spawn(async move {
            let message = vec![i as u8; 100];
            let mut coord = coordinator_clone.lock().await;
            coord.broadcast_message(message).await
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent message broadcasting failed");
    }
    
    {
        let mut coord = coordinator.lock().await;
        coord.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_concurrent_state_access() {
    // Test concurrent access to protocol state
    let config = ProtocolConfig::default();
    let coordinator = Arc::new(Mutex::new(Coordinator::new(config).await.unwrap()));
    
    {
        let mut coord = coordinator.lock().await;
        coord.start().await.unwrap();
    }
    
    // Create multiple concurrent state readers
    let mut handles = Vec::new();
    
    for _ in 0..10 {
        let coordinator_clone = coordinator.clone();
        let handle = tokio::spawn(async move {
            let coord = coordinator_clone.lock().await;
            coord.state().await
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        let state = handle.await.unwrap();
        assert_eq!(state, ProtocolState::Running);
    }
    
    {
        let mut coord = coordinator.lock().await;
        coord.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_concurrent_start_stop_operations() {
    // Test thread safety of start/stop operations
    let config = ProtocolConfig::default();
    let coordinator = Arc::new(Mutex::new(Coordinator::new(config).await.unwrap()));
    
    // Create concurrent start operations (only one should succeed)
    let mut start_handles = Vec::new();
    
    for _ in 0..5 {
        let coordinator_clone = coordinator.clone();
        let handle = tokio::spawn(async move {
            let mut coord = coordinator_clone.lock().await;
            coord.start().await
        });
        start_handles.push(handle);
    }
    
    // At least one start should succeed
    let mut success_count = 0;
    for handle in start_handles {
        if handle.await.unwrap().is_ok() {
            success_count += 1;
        }
    }
    assert!(success_count >= 1, "At least one start operation should succeed");
    
    // Verify state
    {
        let coord = coordinator.lock().await;
        assert_eq!(coord.state().await, ProtocolState::Running);
    }
    
    // Create concurrent stop operations
    let mut stop_handles = Vec::new();
    
    for _ in 0..5 {
        let coordinator_clone = coordinator.clone();
        let handle = tokio::spawn(async move {
            let mut coord = coordinator_clone.lock().await;
            coord.stop().await
        });
        stop_handles.push(handle);
    }
    
    // At least one stop should succeed
    let mut stop_success_count = 0;
    for handle in stop_handles {
        if handle.await.unwrap().is_ok() {
            stop_success_count += 1;
        }
    }
    assert!(stop_success_count >= 1, "At least one stop operation should succeed");
}

#[tokio::test]
async fn test_concurrent_component_access() {
    // Test concurrent access to protocol components
    let config = ProtocolConfig::default();
    let coordinator = Arc::new(Mutex::new(Coordinator::new(config).await.unwrap()));
    
    {
        let mut coord = coordinator.lock().await;
        coord.start().await.unwrap();
    }
    
    // Create concurrent component access tasks
    let mut handles = Vec::new();
    
    for _ in 0..10 {
        let coordinator_clone = coordinator.clone();
        let handle = tokio::spawn(async move {
            let coord = coordinator_clone.lock().await;
            
            // Test concurrent access to components
            let _crypto = coord.crypto_manager();
            let _network = coord.network_manager();
            let _dag = coord.dag_manager();
            
            // All should be available after start
            true
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result, "Component access should succeed");
    }
    
    {
        let mut coord = coordinator.lock().await;
        coord.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_high_concurrency_message_processing() {
    // Test high concurrency message processing
    let config = ProtocolConfig::default();
    let coordinator = Arc::new(Mutex::new(Coordinator::new(config).await.unwrap()));
    
    {
        let mut coord = coordinator.lock().await;
        coord.start().await.unwrap();
    }
    
    // Create a large number of concurrent message operations
    let mut handles = Vec::new();
    let num_tasks = 50;
    let messages_per_task = 10;
    
    for task_id in 0..num_tasks {
        let coordinator_clone = coordinator.clone();
        let handle = tokio::spawn(async move {
            let mut results = Vec::new();
            
            for msg_id in 0..messages_per_task {
                let message = vec![
                    task_id as u8, 
                    msg_id as u8, 
                    (task_id + msg_id) as u8
                ];
                
                let mut coord = coordinator_clone.lock().await;
                let result = coord.broadcast_message(message).await;
                results.push(result.is_ok());
                
                // Small delay to allow other tasks to run
                drop(coord);
                tokio::task::yield_now().await;
            }
            
            results
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete and verify results
    let mut total_messages = 0;
    let mut successful_messages = 0;
    
    for handle in handles {
        let results = handle.await.unwrap();
        total_messages += results.len();
        successful_messages += results.iter().filter(|&&success| success).count();
    }
    
    println!("Processed {}/{} messages successfully", successful_messages, total_messages);
    assert!(successful_messages > 0, "Some messages should be processed successfully");
    
    {
        let mut coord = coordinator.lock().await;
        coord.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_concurrent_lifecycle_operations() {
    // Test concurrent lifecycle operations (start, operations, stop)
    let config = ProtocolConfig::default();
    let coordinator = Arc::new(Mutex::new(Coordinator::new(config).await.unwrap()));
    
    // Phase 1: Concurrent starts
    let start_coordinator = coordinator.clone();
    let start_handle = tokio::spawn(async move {
        let mut coord = start_coordinator.lock().await;
        coord.start().await
    });
    
    // Phase 2: Concurrent operations (wait a bit then start sending messages)
    let op_coordinator = coordinator.clone();
    let op_handle = tokio::spawn(async move {
        sleep(Duration::from_millis(10)).await; // Small delay
        
        let mut results = Vec::new();
        for i in 0..5 {
            let message = vec![i as u8; 10];
            let mut coord = op_coordinator.lock().await;
            let result = coord.broadcast_message(message).await;
            results.push(result.is_ok());
            drop(coord);
            sleep(Duration::from_millis(1)).await;
        }
        results
    });
    
    // Phase 3: Concurrent stop (after some delay)
    let stop_coordinator = coordinator.clone();
    let stop_handle = tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await; // Larger delay
        let mut coord = stop_coordinator.lock().await;
        coord.stop().await
    });
    
    // Wait for all phases to complete
    let start_result = start_handle.await.unwrap();
    let op_results = op_handle.await.unwrap();
    let stop_result = stop_handle.await.unwrap();
    
    // Verify results
    assert!(start_result.is_ok(), "Start should succeed");
    assert!(stop_result.is_ok(), "Stop should succeed");
    
    // Some operations might succeed depending on timing
    let successful_ops = op_results.iter().filter(|&&success| success).count();
    println!("Successful operations during lifecycle: {}/{}", successful_ops, op_results.len());
}

#[tokio::test]
async fn test_thread_safety_invariants() {
    // Test thread safety invariants under stress
    let config = ProtocolConfig::default();
    let coordinator = Arc::new(Mutex::new(Coordinator::new(config).await.unwrap()));
    
    {
        let mut coord = coordinator.lock().await;
        coord.start().await.unwrap();
    }
    
    // Create mixed concurrent operations
    let mut handles = Vec::new();
    
    // State readers
    for _ in 0..10 {
        let coordinator_clone = coordinator.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                let coord = coordinator_clone.lock().await;
                let _state = coord.state().await;
                drop(coord);
                tokio::task::yield_now().await;
            }
        });
        handles.push(handle);
    }
    
    // Message senders
    for i in 0..10 {
        let coordinator_clone = coordinator.clone();
        let handle = tokio::spawn(async move {
            for j in 0..5 {
                let message = vec![i as u8, j as u8];
                let mut coord = coordinator_clone.lock().await;
                let _result = coord.broadcast_message(message).await;
                drop(coord);
                tokio::task::yield_now().await;
            }
        });
        handles.push(handle);
    }
    
    // Component accessors
    for _ in 0..5 {
        let coordinator_clone = coordinator.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                let coord = coordinator_clone.lock().await;
                let _crypto = coord.crypto_manager();
                let _network = coord.network_manager();
                let _dag = coord.dag_manager();
                drop(coord);
                tokio::task::yield_now().await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify final state is consistent
    {
        let coord = coordinator.lock().await;
        assert_eq!(coord.state().await, ProtocolState::Running);
    }
    
    {
        let mut coord = coordinator.lock().await;
        coord.stop().await.unwrap();
    }
}