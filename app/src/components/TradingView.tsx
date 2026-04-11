import { useState } from 'react';
import { useMarketData } from '../hooks/useMarketData';
import { MarketStats } from './MarketStats';
import { CandlestickChart } from './CandlestickChart';
import { OrderBook } from './OrderBook';
import { TradesPanel } from './TradesPanel';

type RightPanelTab = 'book' | 'trades';
type Timeframe = '1h' | '4h' | '1d' | '1w';
type ChartType = 'candlestick' | 'line';

export function TradingView() {
  const { candles, latestCandle } = useMarketData();
  const [activeTab, setActiveTab] = useState<RightPanelTab>('book');
  const [timeframe, setTimeframe] = useState<Timeframe>('1h');
  const [chartType, setChartType] = useState<ChartType>('candlestick');
  
  const previousClose = candles.length > 1 ? candles[candles.length - 2].close : null;

  return (
    <div className="flex flex-col h-screen bg-[#0a0e14]">
      {/* Market Stats Bar with Controls */}
      <MarketStats 
        candle={latestCandle} 
        previousClose={previousClose}
        timeframe={timeframe}
        chartType={chartType}
        onTimeframeChange={(tf) => setTimeframe(tf as Timeframe)}
        onChartTypeChange={(type) => setChartType(type as ChartType)}
      />
      
      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left: Chart Area */}
        <div className="flex-1 flex flex-col border-r border-[#1e2530]">
          {/* Chart */}
          <div className="flex-1 bg-[#0a0e14]">
            <CandlestickChart candles={candles} chartType={chartType} timeframe={timeframe} />
          </div>
        </div>
        
        {/* Right: Order Book / Trades */}
        <div className="w-[420px] flex flex-col bg-[#0d1117]">
          {/* Tabs - Clear and prominent */}
          <div className="flex border-b border-[#1e2530] bg-[#0a0e14]">
            <button
              onClick={() => setActiveTab('book')}
              className={`flex-1 px-6 py-3 text-sm font-semibold transition-all relative ${
                activeTab === 'book'
                  ? 'text-white'
                  : 'text-gray-500 hover:text-gray-300'
              }`}
            >
              Order Book
              {activeTab === 'book' && (
                <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-[#ffffff]" />
              )}
            </button>
            <button
              onClick={() => setActiveTab('trades')}
              className={`flex-1 px-6 py-3 text-sm font-semibold transition-all relative ${
                activeTab === 'trades'
                  ? 'text-white'
                  : 'text-gray-500 hover:text-gray-300'
              }`}
            >
              Recent Trades
              {activeTab === 'trades' && (
                <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-[#ffffff]" />
              )}
            </button>
          </div>
          
          {/* Panel Content */}
          <div className="flex-1 overflow-hidden">
            {activeTab === 'book' ? (
              <OrderBook candle={latestCandle} />
            ) : (
              <TradesPanel candle={latestCandle} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
