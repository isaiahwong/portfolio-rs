import React from 'react'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { PnlDisplay } from '@/components/PnlDisplay'
import type { Execution, PositionSnapshot } from '../api'

function formatInstrument(inst: { symbol: string; venue: string }) {
  return `${inst.symbol} (${inst.venue})`
}

function ExecutionRow({ exec }: { exec: Execution }) {
  const side = exec.side === 'Buy' ? 'Buy' : 'Sell'
  return (
    <TableRow>
      <TableCell>{formatInstrument(exec.instrument_id)}</TableCell>
      <TableCell>{side}</TableCell>
      <TableCell>{exec.qty}</TableCell>
      <TableCell>{exec.px}</TableCell>
    </TableRow>
  )
}

interface PositionSnapshotTableProps {
  positions: PositionSnapshot[]
  expandedId: string | null
  onExpand: (id: string | null) => void
  emptyMessage: string
  currency?: string
}

export function PositionSnapshotTable({
  positions,
  expandedId,
  onExpand,
  emptyMessage,
  currency,
}: PositionSnapshotTableProps) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className="w-8" aria-label="Expand" />
          <TableHead>Instrument</TableHead>
          <TableHead className="hidden sm:table-cell">Side</TableHead>
          <TableHead>Qty</TableHead>
          <TableHead className="hidden md:table-cell">Avg Px</TableHead>
          <TableHead className="hidden md:table-cell">Realized PnL</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {positions.length === 0 ? (
          <TableRow>
            <TableCell colSpan={6} className="h-24 text-center text-muted-foreground">
              {emptyMessage}
            </TableCell>
          </TableRow>
        ) : (
          positions.map((pos) => {
            const inst = pos.position_id.instrument_id
            const isExpanded = expandedId === pos.id
            return (
              <React.Fragment key={pos.id}>
                <TableRow
                  className="cursor-pointer"
                  onClick={() => onExpand(isExpanded ? null : pos.id)}
                >
                  <TableCell className="w-8">
                    {isExpanded ? (
                      <ChevronDown className="h-4 w-4" />
                    ) : (
                      <ChevronRight className="h-4 w-4" />
                    )}
                  </TableCell>
                  <TableCell className="font-medium">{formatInstrument(inst)}</TableCell>
                  <TableCell className="hidden sm:table-cell">{pos.position_side}</TableCell>
                  <TableCell>{pos.qty}</TableCell>
                  <TableCell className="hidden md:table-cell">{pos.avg_px.toFixed(2)}</TableCell>
                  <TableCell className="hidden md:table-cell">
                    <PnlDisplay value={pos.realized_pnl} currency={currency} variant="cell" />
                  </TableCell>
                </TableRow>
                {isExpanded && pos.executions.length > 0 && (
                  <TableRow>
                    <TableCell colSpan={6} className="bg-muted/30 p-0">
                      <div className="overflow-x-auto px-4 py-2">
                        <h5 className="mb-2 text-sm font-medium text-muted-foreground">Executions</h5>
                        <Table>
                          <TableHeader>
                            <TableRow>
                              <TableHead>Instrument</TableHead>
                              <TableHead>Side</TableHead>
                              <TableHead>Qty</TableHead>
                              <TableHead>Price</TableHead>
                            </TableRow>
                          </TableHeader>
                          <TableBody>
                            {pos.executions.map((exec) => (
                              <ExecutionRow key={exec.id} exec={exec} />
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
