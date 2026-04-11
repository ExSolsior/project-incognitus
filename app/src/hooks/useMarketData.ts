import { useState, useEffect, useCallback } from 'react';
import type { MarketData, Candle } from '../types/market';
import { mockWebSocket } from '../services/mockWebSocket';

const MAX_CANDLES = 100; // Keep last 100 candles for chart

export function useMarketData() {
  const [candles, setCandles] = useState<Candle[]>([]);
  const [latestCandle, setLatestCandle] = useState<Candle | null>(null);

  const handleUpdate = useCallback((data: MarketData) => {
    const newCandle = data.candles[0];
    
    setLatestCandle(newCandle);
    
    setCandles(prev => {
      const updated = [...prev, newCandle];
      // Keep only last MAX_CANDLES
      return updated.slice(-MAX_CANDLES);
    });

    // Dispatch event for performance monitor
    window.dispatchEvent(new Event('market-update'));
  }, []);

  useEffect(() => {
    const unsubscribe = mockWebSocket.subscribe(handleUpdate);
    return unsubscribe;
  }, [handleUpdate]);

  return { candles, latestCandle };
}
