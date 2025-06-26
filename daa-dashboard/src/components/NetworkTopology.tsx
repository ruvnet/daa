
import React, { useEffect, useRef } from 'react';

const NetworkTopology = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.width = canvas.offsetWidth * 2;
    const height = canvas.height = canvas.offsetHeight * 2;
    ctx.scale(2, 2);

    // Network nodes
    const nodes = [
      { x: 100, y: 100, size: 12, type: 'orchestrator', connections: 5 },
      { x: 250, y: 80, size: 8, type: 'agent', connections: 3 },
      { x: 180, y: 180, size: 10, type: 'coordinator', connections: 4 },
      { x: 320, y: 160, size: 6, type: 'agent', connections: 2 },
      { x: 80, y: 220, size: 7, type: 'agent', connections: 2 },
      { x: 280, y: 240, size: 9, type: 'storage', connections: 3 },
      { x: 150, y: 280, size: 8, type: 'ml-node', connections: 4 },
      { x: 350, y: 50, size: 5, type: 'agent', connections: 1 },
    ];

    const connections = [
      [0, 1], [0, 2], [0, 4], [1, 2], [1, 3], [2, 4], [2, 5], [2, 6], [3, 5], [5, 6]
    ];

    let animationFrame = 0;

    const animate = () => {
      ctx.clearRect(0, 0, width / 2, height / 2);
      
      // Set dark background
      ctx.fillStyle = '#000000';
      ctx.fillRect(0, 0, width / 2, height / 2);

      // Draw connections
      ctx.strokeStyle = '#10b981';
      ctx.lineWidth = 1;
      ctx.globalAlpha = 0.4 + 0.2 * Math.sin(animationFrame * 0.02);
      
      connections.forEach(([from, to]) => {
        const nodeA = nodes[from];
        const nodeB = nodes[to];
        
        ctx.beginPath();
        ctx.moveTo(nodeA.x, nodeA.y);
        ctx.lineTo(nodeB.x, nodeB.y);
        ctx.stroke();
        
        // Data flow animation
        const progress = (animationFrame * 0.01) % 1;
        const flowX = nodeA.x + (nodeB.x - nodeA.x) * progress;
        const flowY = nodeA.y + (nodeB.y - nodeA.y) * progress;
        
        ctx.fillStyle = '#10b981';
        ctx.globalAlpha = 0.8;
        ctx.beginPath();
        ctx.arc(flowX, flowY, 2, 0, Math.PI * 2);
        ctx.fill();
      });

      // Draw nodes
      ctx.globalAlpha = 1;
      nodes.forEach(node => {
        const pulse = 1 + 0.2 * Math.sin(animationFrame * 0.05 + node.x * 0.01);
        
        // Node glow
        ctx.shadowColor = '#10b981';
        ctx.shadowBlur = 10;
        
        // Node colors by type
        switch (node.type) {
          case 'orchestrator':
            ctx.fillStyle = '#10b981';
            break;
          case 'coordinator':
            ctx.fillStyle = '#3b82f6';
            break;
          case 'storage':
            ctx.fillStyle = '#f59e0b';
            break;
          case 'ml-node':
            ctx.fillStyle = '#8b5cf6';
            break;
          default:
            ctx.fillStyle = '#6b7280';
        }
        
        ctx.beginPath();
        ctx.arc(node.x, node.y, node.size * pulse, 0, Math.PI * 2);
        ctx.fill();
        
        // Node border
        ctx.shadowBlur = 0;
        ctx.strokeStyle = '#10b981';
        ctx.lineWidth = 2;
        ctx.stroke();
        
        // Connection count
        ctx.fillStyle = '#10b981';
        ctx.font = '10px monospace';
        ctx.textAlign = 'center';
        ctx.fillText(node.connections.toString(), node.x, node.y + 3);
      });

      animationFrame++;
      requestAnimationFrame(animate);
    };

    animate();

    return () => {
      // Cleanup handled by useEffect dependencies
    };
  }, []);

  return (
    <div className="relative">
      <canvas 
        ref={canvasRef}
        className="w-full h-64 rounded-lg border border-green-500/20 bg-black"
        style={{ width: '100%', height: '256px' }}
      />
      <div className="absolute top-4 right-4 bg-black/80 p-3 rounded-lg border border-green-500/20">
        <div className="text-xs text-green-400/70 space-y-1">
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 bg-green-400 rounded-full"></div>
            <span>Orchestrator</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 bg-blue-400 rounded-full"></div>
            <span>Coordinator</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 bg-yellow-400 rounded-full"></div>
            <span>Storage</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 bg-purple-400 rounded-full"></div>
            <span>ML Node</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 bg-gray-400 rounded-full"></div>
            <span>Agent</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default NetworkTopology;
