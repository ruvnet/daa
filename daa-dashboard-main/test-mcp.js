// Simple test script to verify MCP integration
import { daaMcpClient } from './dist/assets/index-WlSU09-R.js';

console.log('Testing MCP Connection...');

try {
  const health = await daaMcpClient.healthCheck();
  console.log('MCP Health Check:', health);
  
  if (health.status === 'healthy') {
    console.log('✅ MCP client is healthy');
    
    // Try to get DAA status
    try {
      const status = await daaMcpClient.getStatus();
      console.log('✅ DAA Status:', status);
    } catch (statusError) {
      console.log('⚠️ DAA Status failed:', statusError.message);
    }
    
    // Try to list tools
    try {
      const tools = await daaMcpClient.listTools();
      console.log('✅ Available Tools:', tools);
    } catch (toolsError) {
      console.log('⚠️ List Tools failed:', toolsError.message);
    }
  } else {
    console.log('❌ MCP client is unhealthy:', health.details);
  }
} catch (error) {
  console.log('❌ MCP Connection failed:', error.message);
}