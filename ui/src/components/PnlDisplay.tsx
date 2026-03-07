import { cn } from '@/lib/utils'

interface PnlDisplayProps {
  value: number
  currency?: string
  variant?: 'badge' | 'cell'
  className?: string
}

export function PnlDisplay({ value, currency, variant = 'badge', className }: PnlDisplayProps) {
  const isPositive = value >= 0
  const formatted = `${value >= 0 ? '+' : ''}${value.toFixed(2)}${currency ? ` ${currency}` : ''}`

  return (
    <span
      className={cn(
        'inline-block rounded text-sm',
        isPositive ? 'bg-emerald-500/20 text-emerald-400' : 'bg-red-500/20 text-red-400',
        variant === 'badge' && 'px-2 py-0.5',
        variant === 'cell' && 'px-1.5 py-0.5',
        className
      )}
    >
      {formatted}
    </span>
  )
}
