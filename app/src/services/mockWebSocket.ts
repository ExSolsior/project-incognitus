import type { MarketData, Candle } from '../types/market';

export class MockWebSocketService {
  private intervalId: number | null = null;
  private callbacks: Set<(data: MarketData) => void> = new Set();
  private basePrice = 73225.2;
  private currentCandle: Candle;

  constructor() {
    this.currentCandle = this.generateCandle(Date.now());
  }

  private generateCandle(timestamp: number): Candle {
    // Generate realistic price movements
    const volatility = 0.0005; // 0.05% volatility
    const change = (Math.random() - 0.5) * this.basePrice * volatility;
    
    const open = this.basePrice;
    const close = this.basePrice + change;
    const high = Math.max(open, close) + Math.random() * Math.abs(change);
    const low = Math.min(open, close) - Math.random() * Math.abs(change);
    
    this.basePrice = close; // Update base price for next candle
    
    const volume = 0.5 + Math.random() * 2; // 0.5 to 2.5 BTC
    const buyRatio = 0.4 + Math.random() * 0.2; // 40-60% buy volume
    
    return {
      open_time: timestamp,
      open,
      high,
      low,
      close,
      volume,
      turnover: volume * close,
      trade_count: Math.floor(10 + Math.random() * 50),
      buy_volume: volume * buyRatio,
      sell_volume: volume * (1 - buyRatio),
      buy_turnover: volume * buyRatio * close,
      sell_turnover: volume * (1 - buyRatio) * close,
      buy_trade_count: Math.floor(5 + Math.random() * 25),
      sell_trade_count: Math.floor(5 + Math.random() * 25),
    };
  }

  subscribe(callback: (data: MarketData) => void): () => void {
    this.callbacks.add(callback);
    
    // Start interval if first subscriber
    if (this.callbacks.size === 1) {
      this.start();
    }
    
    // Return unsubscribe function
    return () => {
      this.callbacks.delete(callback);
      if (this.callbacks.size === 0) {
        this.stop();
      }
    };
  }

  private start(): void {
    // Send updates 10 times per second (every 100ms)
    this.intervalId = window.setInterval(() => {
      const data: MarketData = {
        market: 'BTC/USD',
        resolution: '1h',
        candles: [this.generateCandle(Date.now())],
      };
      
      this.callbacks.forEach(callback => callback(data));
    }, 100);
  }

  private stop(): void {
    if (this.intervalId !== null) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }
}

export const mockWebSocket = new MockWebSocketService();
