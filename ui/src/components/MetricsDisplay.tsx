import { cn } from '@/lib/utils'

interface MetricsDisplayProps {
  value: number
  currency?: string
  variant?: 'badge' | 'cell' | 'total'
  className?: string
}

export function MetricsDisplay({ value, currency, variant = 'badge', className }: MetricsDisplayProps) {
  const isTotal = variant === 'total'
  const isPositive = value >= 0
  const formatted = isTotal
    ? `${value.toFixed(2)}${currency ? ` ${currency}` : ''}`
    : `${value >= 0 ? '+' : ''}${value.toFixed(2)}${currency ? ` ${currency}` : ''}`

  return (
    <span
      className={cn(
        'inline-block rounded text-sm',
        isTotal && 'bg-primary/10 text-primary font-semibold px-2.5 py-1',
        !isTotal && (isPositive ? 'bg-emerald-500/20 text-emerald-400' : 'bg-red-500/20 text-red-400'),
        variant === 'badge' && !isTotal && 'px-2 py-0.5',
        variant === 'cell' && 'px-1.5 py-0.5',
        className
      )}
    >
      {formatted}
    </span>
  )
}
