import React, { useMemo, useState } from 'react'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { cn } from '@/lib/utils'
import { ChevronDown, ChevronRight, ChevronUp } from 'lucide-react'
import { MetricsDisplay } from '@/components/MetricsDisplay'
import { useMarketData } from '@/contexts/MarketDataContext'
import type { PositionSnapshot } from '../types'

interface PositionSnapshotTableProps {
  positions: PositionSnapshot[]
  emptyMessage: string
}

export function PositionSnapshotTable({
  positions,
  emptyMessage,
}: PositionSnapshotTableProps) {
  const { getMidPrice } = useMarketData()
  const [venueSort, setVenueSort] = useState<'asc' | 'desc' | null>('asc')
  const [expanded, setExpanded] = useState<Record<string, boolean>>({})

  const sortedPositions = useMemo(() => {
    if (!venueSort) return positions
    return [...positions].sort((a, b) => {
      const va = a.position_id.instrument_id.venue
      const vb = b.position_id.instrument_id.venue
      const cmp = va.localeCompare(vb)
      return venueSort === 'asc' ? cmp : -cmp
    })
  }, [positions, venueSort])

  function cycleVenueSort() {
    setVenueSort((s) => (s === null ? 'asc' : s === 'asc' ? 'desc' : null))
  }

  function toggleExpand(id: string) {
    setExpanded((prev) => ({ ...prev, [id]: !prev[id] }))
  }

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className="w-8" aria-label="Expand" />
          <TableHead>Symbol</TableHead>
          <TableHead
            className="cursor-pointer select-none hover:bg-muted/50"
            onClick={cycleVenueSort}
          >
            <span className="inline-flex items-center gap-1">
              Venue
              {venueSort === 'asc' && <ChevronUp className="h-4 w-4" />}
              {venueSort === 'desc' && <ChevronDown className="h-4 w-4" />}
            </span>
          </TableHead>
          <TableHead className="hidden sm:table-cell">Side</TableHead>
          <TableHead>Qty</TableHead>
          <TableHead className="hidden md:table-cell">Mid</TableHead>
          <TableHead className="hidden md:table-cell">Avg Px</TableHead>
          <TableHead className="hidden md:table-cell"># Executions</TableHead>
          <TableHead className="hidden md:table-cell">Realized PnL</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {positions.length === 0 ? (
          <TableRow>
            <TableCell colSpan={9} className="h-24 text-center text-muted-foreground">
              {emptyMessage}
            </TableCell>
          </TableRow>
        ) : (
          sortedPositions.map((pos) => {
            const inst = pos.position_id.instrument_id
            const isExpanded = !!expanded[pos.id]
            const mid = getMidPrice(inst)
            return (
              <React.Fragment key={pos.id}>
                <TableRow
                  className="cursor-pointer"
                  onClick={() => toggleExpand(pos.id)}
                >
                  <TableCell className="w-8">
                    {isExpanded ? (
                      <ChevronDown className="h-4 w-4" />
                    ) : (
                      <ChevronRight className="h-4 w-4" />
                    )}
                  </TableCell>
                  <TableCell className="font-medium">{inst.symbol}</TableCell>
                  <TableCell>{inst.venue}</TableCell>
                  <TableCell
                    className={cn(
                      'hidden sm:table-cell font-medium',
                      pos.position_side === 'Long' && 'text-emerald-500',
                      pos.position_side === 'Short' && 'text-red-500'
                    )}
                  >
                    {pos.position_side}
                  </TableCell>
                  <TableCell>{pos.qty}</TableCell>
                  <TableCell className="hidden md:table-cell">
                    {mid != null ? `${mid.toFixed(2)} ${pos.quote_currency.code}` : '—'}
                  </TableCell>
                  <TableCell className="hidden md:table-cell">
                    {pos.avg_px.toFixed(2)} {pos.quote_currency.code}
                  </TableCell>
                  <TableCell className="hidden md:table-cell">{pos.executions.length}</TableCell>
                  <TableCell className="hidden md:table-cell">
                    <MetricsDisplay value={pos.realized_pnl} currency={pos.quote_currency.code} variant="cell" />
                  </TableCell>
                </TableRow>
                {isExpanded && pos.executions.length > 0 && (
                  <TableRow>
                    <TableCell colSpan={9} className="bg-muted/30 p-0">
                      <div className="overflow-x-auto px-4 py-1.5">
                        <h5 className="mb-1 text-xs font-medium text-muted-foreground">Executions</h5>
                        <Table>
                          <TableHeader>
                            <TableRow>
                              <TableHead>Symbol</TableHead>
                              <TableHead>Side</TableHead>
                              <TableHead>Qty</TableHead>
                              <TableHead>Price</TableHead>
                            </TableRow>
                          </TableHeader>
                          <TableBody>
                            {pos.executions.map((exec) => (
                              <TableRow key={exec.id}>
                                <TableCell>{exec.instrument_id.symbol}</TableCell>
                                <TableCell>{exec.side}</TableCell>
                                <TableCell>{exec.qty}</TableCell>
                                <TableCell>{exec.px}</TableCell>
                              </TableRow>
                            ))}
                          </TableBody>
                        </Table>
                      </div>
                    </TableCell>
                  </TableRow>
                )}
              </React.Fragment>
            )
          })
        )}
      </TableBody>
    </Table>
  )
}
