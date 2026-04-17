import { memo, useState, useEffect, useRef } from 'react';
import type { Candle, Trade } from '../types/market';

interface TradesPanelProps {
  candle: Candle | null;
}

function TradesPanelComponent({ candle }: TradesPanelProps) {
  const [trades, setTrades] = useState<Trade[]>([]);
  const [flashingTrades, setFlashingTrades] = useState<Set<string>>(new Set());
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!candle) return;

    const newTrade: Trade = {
      id: `${Date.now()}-${Math.random()}`,
      price: candle.close,
      size: 0.001 + Math.random() * 0.1,
      time: Date.now(), // Use current time for proper ordering
      side: candle.close >= candle.open ? 'buy' : 'sell',
    };

    setTrades(prev => [newTrade, ...prev].slice(0, 50));
    
    setFlashingTrades(new Set([newTrade.id]));
    setTimeout(() => setFlashingTrades(new Set()), 500);

    // Auto-scroll to top when new trade arrives
    if (scrollRef.current) {
      scrollRef.current.scrollTop = 0;
    }
  }, [candle]);

  if (!candle) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="w-8 h-8 border-2 border-[#1e88e5] border-t-transparent rounded-full animate-spin mx-auto mb-3" />
          <div className="text-sm text-gray-500">Loading trades...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-[#0d1117]">
      {/* Header */}
      <div className="grid grid-cols-3 gap-3 px-4 py-3 text-xs font-semibold text-gray-400 border-b border-[#1e2530] bg-[#0a0e14]">
        <div className="text-right">Price (USD)</div>
        <div className="text-right">Size (BTC)</div>
        <div className="text-right">Time</div>
      </div>
      
      {/* Trades list - scrolls to top automatically */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto scrollbar-thin">
        {trades.map((trade) => {
          const isFlashing = flashingTrades.has(trade.id);
          
          return (
            <div
              key={trade.id}
              className={`grid grid-cols-3 gap-3 px-4 py-1.5 transition-all hover:bg-white/5 ${
                isFlashing ? (trade.side === 'buy' ? 'flash-green' : 'flash-red') : ''
              }`}
            >
              <div className={`text-right font-semibold tabular-nums text-sm ${
                trade.side === 'buy' ? 'text-[#26a69a]' : 'text-[#ef5350]'
              }`}>
                {trade.price.toLocaleString('en-US', { minimumFractionDigits: 1, maximumFractionDigits: 1 })}
              </div>
              <div className="text-right text-white tabular-nums text-sm">
                {trade.size.toFixed(5)}
              </div>
              <div className="text-right text-gray-500 tabular-nums text-xs">
                {new Date(trade.time).toLocaleTimeString('en-US', { 
                  hour: '2-digit', 
                  minute: '2-digit',
                  second: '2-digit',
                  hour12: false 
                })}
              </div>
            </div>
          );
        })}
      </div>

      {/* Footer Stats */}
      <div className="px-4 py-3 border-t border-[#1e2530] bg-[#0a0e14]">
        <div className="flex items-center justify-between text-xs">
          <div className="flex items-center gap-2">
            <div className="w-1.5 h-1.5 rounded-full bg-[#26a69a]" />
            <span className="text-gray-400">{trades.length} recent trades</span>
          </div>
          <span className="text-gray-500">Live updates</span>
        </div>
      </div>
    </div>
  );
}

export const TradesPanel = memo(TradesPanelComponent);
