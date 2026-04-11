import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { MockWebSocketService } from '../services/mockWebSocket';
import type { MarketData } from '../types/market';

describe('MockWebSocketService', () => {
  let service: MockWebSocketService;
  
  beforeEach(() => {
    vi.useFakeTimers();
    service = new MockWebSocketService();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should send 10 updates per second', () => {
    const callback = vi.fn();
    service.subscribe(callback);

    // Advance time by 1 second
    vi.advanceTimersByTime(1000);

    // Should have received 10 updates (every 100ms)
    expect(callback).toHaveBeenCalledTimes(10);
  });

  it('should send data in correct format', () => {
    const callback = vi.fn();
    service.subscribe(callback);

    vi.advanceTimersByTime(100);

    expect(callback).toHaveBeenCalledWith(
      expect.objectContaining({
        market: 'BTC/USD',
        resolution: '1h',
        candles: expect.arrayContaining([
          expect.objectContaining({
            open_time: expect.any(Number),
            open: expect.any(Number),
            high: expect.any(Number),
            low: expect.any(Number),
            close: expect.any(Number),
            volume: expect.any(Number),
            turnover: expect.any(Number),
            trade_count: expect.any(Number),
            buy_volume: expect.any(Number),
            sell_volume: expect.any(Number),
            buy_turnover: expect.any(Number),
            sell_turnover: expect.any(Number),
            buy_trade_count: expect.any(Number),
            sell_trade_count: expect.any(Number),
          }),
        ]),
      })
    );
  });

  it('should generate realistic candle data', () => {
    const callback = vi.fn<[MarketData], void>();
    service.subscribe(callback);

    vi.advanceTimersByTime(100);

    const data = callback.mock.calls[0][0];
    const candle = data.candles[0];

    // High should be >= max(open, close)
    expect(candle.high).toBeGreaterThanOrEqual(Math.max(candle.open, candle.close));
    
    // Low should be <= min(open, close)
    expect(candle.low).toBeLessThanOrEqual(Math.min(candle.open, candle.close));
    
    // Volume should be positive
    expect(candle.volume).toBeGreaterThan(0);
    
    // Buy + sell volume should equal total volume
    expect(candle.buy_volume + candle.sell_volume).toBeCloseTo(candle.volume, 5);
  });

  it('should stop sending updates when all subscribers unsubscribe', () => {
    const callback1 = vi.fn();
    const callback2 = vi.fn();
    
    const unsubscribe1 = service.subscribe(callback1);
    const unsubscribe2 = service.subscribe(callback2);

    vi.advanceTimersByTime(100);
    expect(callback1).toHaveBeenCalledTimes(1);
    expect(callback2).toHaveBeenCalledTimes(1);

    unsubscribe1();
    unsubscribe2();

    callback1.mockClear();
    callback2.mockClear();

    vi.advanceTimersByTime(100);
    expect(callback1).not.toHaveBeenCalled();
    expect(callback2).not.toHaveBeenCalled();
  });

  it('should handle multiple subscribers', () => {
    const callback1 = vi.fn();
    const callback2 = vi.fn();
    const callback3 = vi.fn();
    
    service.subscribe(callback1);
    service.subscribe(callback2);
    service.subscribe(callback3);

    vi.advanceTimersByTime(100);

    expect(callback1).toHaveBeenCalledTimes(1);
    expect(callback2).toHaveBeenCalledTimes(1);
    expect(callback3).toHaveBeenCalledTimes(1);

    // All should receive the same data
    expect(callback1.mock.calls[0][0]).toEqual(callback2.mock.calls[0][0]);
    expect(callback2.mock.calls[0][0]).toEqual(callback3.mock.calls[0][0]);
  });
});
