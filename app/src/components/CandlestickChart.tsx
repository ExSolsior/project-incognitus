import { memo, useEffect, useRef, useState } from 'react';
import type { Candle } from '../types/market';

interface CandlestickChartProps {
  candles: Candle[];
  chartType: 'candlestick' | 'line';
  timeframe: string;
}

interface CrosshairData {
  x: number;
  y: number;
  candle: Candle | null;
  price: number;
}

function CandlestickChartComponent({ candles, chartType, timeframe }: CandlestickChartProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [crosshair, setCrosshair] = useState<CrosshairData | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container || candles.length === 0) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = container.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    canvas.style.width = `${rect.width}px`;
    canvas.style.height = `${rect.height}px`;
    ctx.scale(dpr, dpr);

    const width = rect.width;
    const height = rect.height;
    const padding = { top: 10, right: 80, bottom: 7, left: 5 };
    const chartHeight = height - padding.top - padding.bottom;
    const chartWidth = width - padding.left - padding.right;

    // Deep black background
    ctx.fillStyle = '#0a0e14';
    ctx.fillRect(0, 0, width, height);

    const prices = candles.flatMap(c => [c.high, c.low]);
    const maxPrice = Math.max(...prices);
    const minPrice = Math.min(...prices);
    const priceRange = maxPrice - minPrice;
    const priceScale = chartHeight / priceRange;

    const maxVolume = Math.max(...candles.map(c => c.volume));
    const volumeHeight = 60;
    const volumeScale = volumeHeight / maxVolume;

    const candleWidth = Math.max(4, chartWidth / candles.length - 3);
    const candleSpacing = chartWidth / candles.length;

    // Grid lines - subtle
    ctx.strokeStyle = '#1e2530';
    ctx.lineWidth = 1;
    
    for (let i = 0; i <= 6; i++) {
      const y = padding.top + (chartHeight / 6) * i;
      ctx.beginPath();
      ctx.moveTo(padding.left, y);
      ctx.lineTo(width - padding.right, y);
      ctx.stroke();

      // Price labels - bright and clear
      const price = maxPrice - (priceRange / 6) * i;
      ctx.fillStyle = '#b0b8c1';
      ctx.font = 'bold 12px monospace';
      ctx.textAlign = 'left';
      ctx.fillText(
        `$${price.toLocaleString('en-US', { minimumFractionDigits: 1, maximumFractionDigits: 1 })}`, 
        width - padding.right + 8, 
        y + 4
      );
    }

    // Vertical grid lines
    const verticalLines = 8;
    for (let i = 0; i <= verticalLines; i++) {
      const x = padding.left + (chartWidth / verticalLines) * i;
      ctx.strokeStyle = '#1e2530';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(x, padding.top);
      ctx.lineTo(x, height - padding.bottom);
      ctx.stroke();
    }

    if (chartType === 'candlestick') {
      // Candlesticks - SOFTER COLORS
      candles.forEach((candle, i) => {
        const x = padding.left + i * candleSpacing + candleSpacing / 2;
        const isGreen = candle.close >= candle.open;
        const color = isGreen ? '#26a69a' : '#ef5350'; // Softer teal and red

        // Wick
        const highY = padding.top + (maxPrice - candle.high) * priceScale;
        const lowY = padding.top + (maxPrice - candle.low) * priceScale;
        
        ctx.strokeStyle = color;
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(x, highY);
        ctx.lineTo(x, lowY);
        ctx.stroke();

        // Body - solid and visible
        const openY = padding.top + (maxPrice - candle.open) * priceScale;
        const closeY = padding.top + (maxPrice - candle.close) * priceScale;
        const bodyTop = Math.min(openY, closeY);
        const bodyHeight = Math.max(2, Math.abs(closeY - openY));

        ctx.fillStyle = color;
        ctx.fillRect(x - candleWidth / 2, bodyTop, candleWidth, bodyHeight);

        // Volume bar - SOFTER
        const volumeY = height - padding.bottom - candle.volume * volumeScale;
        ctx.fillStyle = isGreen ? 'rgba(38, 166, 154, 0.4)' : 'rgba(239, 83, 80, 0.4)';
        ctx.fillRect(x - candleWidth / 2, volumeY, candleWidth, candle.volume * volumeScale);
      });
    } else {
      // Line chart
      ctx.strokeStyle = '#1e88e5';
      ctx.lineWidth = 2;
      ctx.beginPath();
      
      candles.forEach((candle, i) => {
        const x = padding.left + i * candleSpacing + candleSpacing / 2;
        const y = padding.top + (maxPrice - candle.close) * priceScale;
        
        if (i === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      });
      
      ctx.stroke();

      // Fill area under line
      ctx.lineTo(padding.left + (candles.length - 1) * candleSpacing + candleSpacing / 2, height - padding.bottom);
      ctx.lineTo(padding.left + candleSpacing / 2, height - padding.bottom);
      ctx.closePath();
      
      const gradient = ctx.createLinearGradient(0, padding.top, 0, height - padding.bottom);
      gradient.addColorStop(0, 'rgba(30, 136, 229, 0.3)');
      gradient.addColorStop(1, 'rgba(30, 136, 229, 0.0)');
      ctx.fillStyle = gradient;
      ctx.fill();

      // Volume bars for line chart
      candles.forEach((candle, i) => {
        const x = padding.left + i * candleSpacing + candleSpacing / 2;
        const isGreen = candle.close >= candle.open;
        const volumeY = height - padding.bottom - candle.volume * volumeScale;
        ctx.fillStyle = isGreen ? 'rgba(38, 166, 154, 0.4)' : 'rgba(239, 83, 80, 0.4)';
        ctx.fillRect(x - candleWidth / 2, volumeY, candleWidth, candle.volume * volumeScale);
      });
    }

    // Crosshair
    if (crosshair) {
      ctx.strokeStyle = '#6b7280';
      ctx.lineWidth = 1;
      ctx.setLineDash([5, 5]);
      
      ctx.beginPath();
      ctx.moveTo(crosshair.x, padding.top);
      ctx.lineTo(crosshair.x, height - padding.bottom);
      ctx.stroke();

      ctx.beginPath();
      ctx.moveTo(padding.left, crosshair.y);
      ctx.lineTo(width - padding.right, crosshair.y);
      ctx.stroke();
      ctx.setLineDash([]);

      // Price label
      ctx.fillStyle = '#6b7280';
      ctx.fillRect(width - padding.right, crosshair.y - 12, padding.right - 4, 24);
      ctx.fillStyle = '#ffffff';
      ctx.font = 'bold 12px monospace';
      ctx.textAlign = 'left';
      ctx.fillText(
        `$${crosshair.price.toLocaleString('en-US', { minimumFractionDigits: 1, maximumFractionDigits: 1 })}`,
        width - padding.right + 8,
        crosshair.y + 4
      );
    }

  }, [candles, crosshair, chartType, timeframe]);

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    const container = containerRef.current;
    if (!container || candles.length === 0) return;

    const rect = container.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const padding = { top: 10, right: 80, bottom: 40, left: 10 };
    const chartHeight = rect.height - padding.top - padding.bottom;
    
    const prices = candles.flatMap(c => [c.high, c.low]);
    const maxPrice = Math.max(...prices);
    const minPrice = Math.min(...prices);
    const priceRange = maxPrice - minPrice;

    const price = maxPrice - ((y - padding.top) / chartHeight) * priceRange;

    const chartWidth = rect.width - padding.left - padding.right;
    const candleSpacing = chartWidth / candles.length;
    const candleIndex = Math.floor((x - padding.left) / candleSpacing);
    const candle = candles[candleIndex] || null;

    setCrosshair({ x, y, candle, price });
  };

  return (
    <div 
      ref={containerRef} 
      className="w-full h-full relative"
      onMouseMove={handleMouseMove}
      onMouseLeave={() => setCrosshair(null)}
    >
      <canvas ref={canvasRef} className="w-full h-full" />
      
      {/* Tooltip - clean and readable */}
      {crosshair?.candle && (
        <div 
          className="absolute glass border border-[#1e2530] rounded-lg px-4 py-3 pointer-events-none shadow-2xl"
          style={{
            left: Math.min(crosshair.x + 20, containerRef.current!.clientWidth - 200),
            top: Math.max(20, crosshair.y - 90),
          }}
        >
          <div className="space-y-1.5 text-sm">
            <div className="text-gray-400 text-xs font-semibold mb-2">
              {new Date(crosshair.candle.open_time).toLocaleString('en-US', {
                month: 'short',
                day: 'numeric',
                hour: '2-digit',
                minute: '2-digit',
              })}
            </div>
            {chartType === 'candlestick' ? (
              <>
                <div className="flex justify-between gap-6">
                  <span className="text-gray-500">Open</span>
                  <span className="text-white font-semibold tabular-nums">
                    ${crosshair.candle.open.toFixed(1)}
                  </span>
                </div>
                <div className="flex justify-between gap-6">
                  <span className="text-gray-500">High</span>
                  <span className="text-[#26a69a] font-semibold tabular-nums">
                    ${crosshair.candle.high.toFixed(1)}
                  </span>
                </div>
                <div className="flex justify-between gap-6">
                  <span className="text-gray-500">Low</span>
                  <span className="text-[#ef5350] font-semibold tabular-nums">
                    ${crosshair.candle.low.toFixed(1)}
                  </span>
                </div>
                <div className="flex justify-between gap-6">
                  <span className="text-gray-500">Close</span>
                  <span className="text-white font-semibold tabular-nums">
                    ${crosshair.candle.close.toFixed(1)}
                  </span>
                </div>
              </>
            ) : (
              <div className="flex justify-between gap-6">
                <span className="text-gray-500">Price</span>
                <span className="text-[#1e88e5] font-semibold tabular-nums">
                  ${crosshair.candle.close.toFixed(1)}
                </span>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

export const CandlestickChart = memo(CandlestickChartComponent);
