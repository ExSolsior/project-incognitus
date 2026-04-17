import { useState, useEffect, useRef } from 'react';

export function PerformanceMonitor() {
  const [fps, setFps] = useState(0);
  const [updateRate, setUpdateRate] = useState(0);
  const frameCountRef = useRef(0);
  const updateCountRef = useRef(0);
  const lastTimeRef = useRef(Date.now());

  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now();
      const elapsed = (now - lastTimeRef.current) / 1000;
      
      setFps(Math.round(frameCountRef.current / elapsed));
      setUpdateRate(Math.round(updateCountRef.current / elapsed));
      
      frameCountRef.current = 0;
      updateCountRef.current = 0;
      lastTimeRef.current = now;
    }, 1000);

    const animationFrame = () => {
      frameCountRef.current++;
      requestAnimationFrame(animationFrame);
    };
    requestAnimationFrame(animationFrame);

    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const handler = () => {
      updateCountRef.current++;
    };
    window.addEventListener('market-update', handler);
    return () => window.removeEventListener('market-update', handler);
  }, []);

  return (
    <div className="fixed bottom-6 left-6 glass border border-[#1e2530] rounded-lg px-4 py-3 shadow-2xl">
      <div className="flex items-center gap-6">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-[#26a69a]" />
          <span className="text-xs font-semibold text-gray-400">LIVE</span>
        </div>
        <div className="flex items-center gap-2 font-mono text-sm">
          <span className="text-gray-500">FPS</span>
          <span className={`font-bold ${fps >= 50 ? 'text-[#26a69a]' : 'text-[#ff9800]'}`}>
            {fps}
          </span>
        </div>
        <div className="w-px h-4 bg-[#1e2530]" />
        <div className="flex items-center gap-2 font-mono text-sm">
          <span className="text-gray-500">Updates</span>
          <span className="font-bold text-gray-400">{updateRate}/s</span>
        </div>
      </div>
    </div>
  );
}
