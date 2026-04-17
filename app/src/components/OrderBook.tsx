import { memo, useMemo, useState } from 'react';
import type { Candle } from '../types/market';
import type { OrderBookLevel } from '../types/market';

interface OrderBookProps {
  candle: Candle | null;
}

function OrderBookComponent({ candle }: OrderBookProps) {
  const [hoveredLevel, setHoveredLevel] = useState<string | null>(null);

  const { asks, bids, maxTotal, currentPrice } = useMemo(() => {
    if (!candle) {
      return { asks: [], bids: [], maxTotal: 0, currentPrice: 0 };
    }

    const currentPrice = candle.close;
    const spread = currentPrice * 0.0001;
    
    const askLevels: OrderBookLevel[] = [];
    const bidLevels: OrderBookLevel[] = [];
    
    let askTotal = 0;
    let bidTotal = 0;
    
    for (let i = 0; i < 12; i++) {
      const askPrice = currentPrice + spread + (i * spread * 0.5);
      const askSize = 0.001 + Math.random() * 0.5;
      askTotal += askSize;
      askLevels.push({ price: askPrice, size: askSize, total: askTotal });
      
      const bidPrice = currentPrice - spread - (i * spread * 0.5);
      const bidSize = 0.001 + Math.random() * 0.5;
      bidTotal += bidSize;
      bidLevels.push({ price: bidPrice, size: bidSize, total: bidTotal });
    }
    
    const maxTotal = Math.max(askTotal, bidTotal);
    
    return { asks: askLevels, bids: bidLevels, maxTotal, currentPrice };
  }, [candle]);

  if (!candle) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="w-8 h-8 border-2 border-[#1e88e5] border-t-transparent rounded-full animate-spin mx-auto mb-3" />
          <div className="text-sm text-gray-500">Loading order book...</div>
        </div>
      </div>
    );
  }

  const renderLevel = (level: OrderBookLevel, type: 'ask' | 'bid', idx: number) => {
    const key = `${level.price.toFixed(1)}`;
    const isHovered = hoveredLevel === key;
    const depthPercent = (level.total / maxTotal) * 100;
    
    return (
      <div
        key={`${type}-${idx}`}
        className={`relative grid grid-cols-3 gap-3 px-4 py-1.5 cursor-pointer transition-all ${
          isHovered ? 'bg-white/5' : ''
        }`}
        onMouseEnter={() => setHoveredLevel(key)}
        onMouseLeave={() => setHoveredLevel(null)}
      >
        {/* Depth bar - VISIBLE */}
        <div
          className={`absolute right-0 top-0 bottom-0 transition-all ${
            type === 'ask' 
              ? 'bg-[#ef5350]' 
              : 'bg-[#26a69a]'
          }`}
          style={{ 
            width: `${depthPercent}%`,
            opacity: isHovered ? 0.25 : 0.15
          }}
        />
        
        <div className={`relative text-right font-semibold tabular-nums text-sm ${
          type === 'ask' ? 'text-[#ef5350]' : 'text-[#26a69a]'
        }`}>
          {level.price.toLocaleString('en-US', { minimumFractionDigits: 1, maximumFractionDigits: 1 })}
        </div>
        <div className="relative text-right text-white tabular-nums text-sm">
          {level.size.toFixed(5)}
        </div>
        <div className="relative text-right text-gray-500 tabular-nums text-sm">
          {level.total.toFixed(4)}
        </div>
      </div>
    );
  };

  return (
    <div className="flex flex-col h-full bg-[#0d1117]">
      {/* Header */}
      <div className="grid grid-cols-3 gap-3 px-4 py-3 text-xs font-semibold text-gray-400 border-b border-[#1e2530] bg-[#0a0e14]">
        <div className="text-right">Price (USD)</div>
        <div className="text-right">Size (BTC)</div>
        <div className="text-right">Total (BTC)</div>
      </div>
      
      {/* Asks */}
      <div className="flex-1 overflow-hidden">
        <div className="flex flex-col-reverse h-full overflow-y-auto scrollbar-thin">
          {asks.map((level, idx) => renderLevel(level, 'ask', idx))}
        </div>
      </div>
      
      {/* Current Price - PROMINENT */}
      <div className="px-4 py-4 bg-[#0a0e14] border-y border-[#1e2530]">
        <div className="flex items-center justify-between">
          <div>
            <div className={`text-2xl font-bold tabular-nums ${
              candle.close >= candle.open ? 'text-[#26a69a]' : 'text-[#ef5350]'
            }`}>
              ${currentPrice.toLocaleString('en-US', { minimumFractionDigits: 1, maximumFractionDigits: 1 })}
            </div>
          </div>
          <div className="text-right">
            <div className="text-xs text-gray-500">Spread</div>
            <div className="text-sm font-semibold text-white">0.01%</div>
          </div>
        </div>
      </div>
      
      {/* Bids */}
      <div className="flex-1 overflow-hidden">
        <div className="h-full overflow-y-auto scrollbar-thin">
          {bids.map((level, idx) => renderLevel(level, 'bid', idx))}
        </div>
      </div>
    </div>
  );
}

export const OrderBook = memo(OrderBookComponent);
