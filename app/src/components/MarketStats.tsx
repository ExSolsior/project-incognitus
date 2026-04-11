import { memo } from 'react';
import type { Candle } from '../types/market';

interface MarketStatsProps {
  candle: Candle | null;
  previousClose: number | null;
  timeframe: string;
  chartType: string;
  onTimeframeChange: (tf: string) => void;
  onChartTypeChange: (type: string) => void;
}

function MarketStatsComponent({ 
  candle, 
  previousClose, 
  timeframe, 
  chartType, 
  onTimeframeChange, 
  onChartTypeChange 
}: MarketStatsProps) {
  if (!candle) {
    return (
      <div className="bg-[#0d1117] border-b border-[#1e2530] px-6 py-3">
        <div className="flex items-center gap-3">
          <div className="w-2 h-2 rounded-full bg-gray-600 animate-pulse" />
          <span className="text-sm text-gray-500">Connecting to market...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-[#0d1117] border-b border-[#1e2530] px-6 py-4">
      <div className="flex items-center gap-8">
        {/* Market Pair & Live Badge */}
        <div className="flex items-center gap-3">
          <span className="text-xl font-bold text-white">BTC/USD</span>
          <div className="flex items-center gap-1.5 px-2 py-1 bg-[#26a69a]/10 rounded">
            <div className="w-1.5 h-1.5 rounded-full bg-[#26a69a]" />
            <span className="text-xs font-medium text-[#26a69a]">Live</span>
          </div>
        </div>

        {/* Divider */}
        <div className="h-10 w-px bg-[#1e2530]" />

        {/* 24h Stats - Inline Format */}
        <div className="flex items-center gap-6 text-sm">
          <div className="flex items-center gap-2">
            <span className="text-gray-500">24h High</span>
            <span className="font-semibold text-[#26a69a] tabular-nums">
              ${candle.high.toLocaleString('en-US', { maximumFractionDigits: 0 })}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-gray-500">24h Low</span>
            <span className="font-semibold text-[#ef5350] tabular-nums">
              ${candle.low.toLocaleString('en-US', { maximumFractionDigits: 0 })}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-gray-500">Volume</span>
            <span className="font-semibold text-white tabular-nums">
              {candle.volume.toFixed(2)}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-gray-500">Turnover</span>
            <span className="font-semibold text-white tabular-nums">
              ${(candle.turnover / 1000000).toFixed(1)}M
            </span>
          </div>
        </div>

        {/* Spacer to push controls to the right */}
        <div className="flex-1" />

        {/* Controls - Larger Touch Targets */}
        <div className="flex items-center gap-3">
          {/* Timeframe Selector */}
          <div className="flex items-center gap-1 bg-[#0a0e14] rounded-md p-1">
            {['1h', '4h', '1d', '1w'].map((tf) => (
              <button
                key={tf}
                onClick={() => onTimeframeChange(tf)}
                className={`px-3 py-1.5 text-xs font-semibold rounded transition-all ${
                  timeframe === tf
                    ? 'bg-[#2a3441] text-white'
                    : 'text-gray-500 hover:text-gray-300'
                }`}
              >
                {tf.toUpperCase()}
              </button>
            ))}
          </div>

          {/* Chart Type Selector */}
          <div className="flex items-center gap-1 bg-[#0a0e14] rounded-md p-1">
            <button
              onClick={() => onChartTypeChange('candlestick')}
              className={`px-3 py-1.5 text-xs font-semibold rounded transition-all ${
                chartType === 'candlestick'
                  ? 'bg-[#2a3441] text-white'
                  : 'text-gray-500 hover:text-gray-300'
              }`}
            >
              Candles
            </button>
            <button
              onClick={() => onChartTypeChange('line')}
              className={`px-3 py-1.5 text-xs font-semibold rounded transition-all ${
                chartType === 'line'
                  ? 'bg-[#2a3441] text-white'
                  : 'text-gray-500 hover:text-gray-300'
              }`}
            >
              Line
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export const MarketStats = memo(MarketStatsComponent);
