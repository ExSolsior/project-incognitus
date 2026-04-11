// Market data types
export interface Candle {
  open_time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  turnover: number;
  trade_count: number;
  buy_volume: number;
  sell_volume: number;
  buy_turnover: number;
  sell_turnover: number;
  buy_trade_count: number;
  sell_trade_count: number;
}

export interface MarketData {
  market: string;
  resolution: string;
  candles: Candle[];
}

export interface OrderBookLevel {
  price: number;
  size: number;
  total: number;
}

export interface Trade {
  id: string;
  price: number;
  size: number;
  time: number;
  side: 'buy' | 'sell';
}
