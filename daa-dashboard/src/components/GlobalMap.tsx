
import React, { useEffect, useRef } from 'react';

const GlobalMap = () => {
  const mapRef = useRef<HTMLDivElement>(null);

  // Mock data for global nodes
  const nodes = [
    { id: 1, name: 'US-EAST-1', lat: 40.7128, lng: -74.0060, status: 'active', agents: 2840 },
    { id: 2, name: 'US-WEST-1', lat: 37.7749, lng: -122.4194, status: 'active', agents: 1950 },
    { id: 3, name: 'EU-CENTRAL-1', lat: 50.1109, lng: 8.6821, status: 'active', agents: 3200 },
    { id: 4, name: 'ASIA-PACIFIC-1', lat: 35.6762, lng: 139.6503, status: 'active', agents: 2100 },
    { id: 5, name: 'EU-WEST-1', lat: 51.5074, lng: -0.1278, status: 'degraded', agents: 890 },
    { id: 6, name: 'ASIA-SOUTH-1', lat: 19.0760, lng: 72.8777, status: 'active', agents: 1650 },
    { id: 7, name: 'OCEANIA-1', lat: -33.8688, lng: 151.2093, status: 'active', agents: 420 },
  ];

  const connections = [
    { from: 1, to: 3, strength: 0.8 },
    { from: 1, to: 2, strength: 0.9 },
    { from: 3, to: 5, strength: 0.6 },
    { from: 3, to: 4, strength: 0.7 },
    { from: 4, to: 6, strength: 0.8 },
    { from: 4, to: 7, strength: 0.5 },
  ];

  useEffect(() => {
    if (!mapRef.current) return;

    // Simple SVG-based world map visualization
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('width', '100%');
    svg.setAttribute('height', '400');
    svg.setAttribute('viewBox', '0 0 1000 500');
    svg.style.background = 'radial-gradient(ellipse at center, #001a00 0%, #000000 100%)';

    // Draw world outline (simplified)
    const worldPath = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    worldPath.setAttribute('d', 'M100 200 Q200 150 300 200 T500 200 Q600 180 700 200 T900 200 L900 350 Q800 380 700 350 T500 350 Q400 370 300 350 T100 350 Z');
    worldPath.setAttribute('fill', 'none');
    worldPath.setAttribute('stroke', '#10b981');
    worldPath.setAttribute('stroke-width', '1');
    worldPath.setAttribute('opacity', '0.3');
    svg.appendChild(worldPath);

    // Convert lat/lng to SVG coordinates
    const toSVG = (lat: number, lng: number) => ({
      x: ((lng + 180) / 360) * 1000,
      y: ((90 - lat) / 180) * 500
    });

    // Draw connections
    connections.forEach(conn => {
      const fromNode = nodes.find(n => n.id === conn.from);
      const toNode = nodes.find(n => n.id === conn.to);
      if (fromNode && toNode) {
        const from = toSVG(fromNode.lat, fromNode.lng);
        const to = toSVG(toNode.lat, toNode.lng);
        
        const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        line.setAttribute('x1', from.x.toString());
        line.setAttribute('y1', from.y.toString());
        line.setAttribute('x2', to.x.toString());
        line.setAttribute('y2', to.y.toString());
        line.setAttribute('stroke', '#10b981');
        line.setAttribute('stroke-width', (conn.strength * 2).toString());
        line.setAttribute('opacity', (conn.strength * 0.6).toString());
        line.classList.add('animate-pulse');
        svg.appendChild(line);
      }
    });

    // Draw nodes
    nodes.forEach(node => {
      const pos = toSVG(node.lat, node.lng);
      
      // Node circle
      const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
      circle.setAttribute('cx', pos.x.toString());
      circle.setAttribute('cy', pos.y.toString());
      circle.setAttribute('r', Math.max(5, Math.log(node.agents) * 2).toString());
      circle.setAttribute('fill', node.status === 'active' ? '#10b981' : '#f59e0b');
      circle.setAttribute('opacity', '0.8');
      circle.classList.add('animate-pulse');
      svg.appendChild(circle);

      // Node label
      const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      text.setAttribute('x', (pos.x + 15).toString());
      text.setAttribute('y', (pos.y + 5).toString());
      text.setAttribute('fill', '#10b981');
      text.setAttribute('font-size', '12');
      text.setAttribute('font-family', 'monospace');
      text.textContent = `${node.name} (${node.agents})`;
      svg.appendChild(text);
    });

    mapRef.current.appendChild(svg);

    return () => {
      if (mapRef.current && svg.parentNode) {
        mapRef.current.removeChild(svg);
      }
    };
  }, []);

  return (
    <div className="relative">
      <div ref={mapRef} className="w-full h-96 rounded-lg border border-green-500/20" />
      <div className="absolute top-4 right-4 bg-black/80 p-3 rounded-lg border border-green-500/20">
        <div className="text-xs text-green-400/70 space-y-1">
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 bg-green-400 rounded-full animate-pulse"></div>
            <span>Active Nodes ({nodes.filter(n => n.status === 'active').length})</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 bg-yellow-400 rounded-full"></div>
            <span>Degraded Nodes ({nodes.filter(n => n.status === 'degraded').length})</span>
          </div>
          <div className="mt-2 pt-2 border-t border-green-500/20">
            <span>Total Agents: {nodes.reduce((sum, n) => sum + n.agents, 0).toLocaleString()}</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default GlobalMap;
