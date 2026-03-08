import { useState, useEffect, type SubmitEvent } from 'react'
import {
  getPositions,
  getHistory,
  addExecution,
  getPortfolioCalc,
} from '../api'
import type { PositionSnapshot, PortfolioCalc, InstrumentId, Side } from '../types'
import { useMarketData } from '@/contexts/MarketDataContext'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { MetricsDisplay } from '@/components/MetricsDisplay'
import { PositionSnapshotTable } from '@/components/PositionSnapshotTable'

interface PortfolioCardProps {
  id: string
  onRefresh?: () => void
}

interface PortfolioFormState {
  instrument_id: InstrumentId | null
  side: Side
  qty: string
  px: string
}

export function PortfolioCard({ id, onRefresh }: PortfolioCardProps) {
  const { instruments, currencies } = useMarketData()
  const [positions, setPositions] = useState<PositionSnapshot[]>([])
  const [history, setHistory] = useState<PositionSnapshot[]>([])
  const [calc, setCalc] = useState<PortfolioCalc | null>(null)
  const [currency, setCurrency] = useState<string>('USD')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [form, setForm] = useState<PortfolioFormState>({ instrument_id: null, side: 'Buy', qty: '', px: '' })

  useEffect(() => {
    load()
  }, [id, currency])

  async function load() {
    setLoading(true)
    setError(null)
    try {

      const [positionsRes, historyRes, calcRes] = await Promise.all([
        getPositions(id),
        getHistory(id),
        getPortfolioCalc(id, currency).catch(() => null),
      ])

      setPositions(positionsRes)
      setHistory(historyRes)
      setCalc(calcRes)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Unknown error')
    } finally {
      setLoading(false)
    }
  }

  async function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault()
    if (!form.instrument_id) {
      setError('Select an instrument')
      return
    }

    const qty = parseFloat(form.qty)
    const px = parseFloat(form.px)
    if (isNaN(qty) || isNaN(px) || qty <= 0 || px <= 0) {
      setError('Invalid qty or px')
      return
    }

    try {
      await addExecution(id, {
        instrument_id: form.instrument_id,
        side: form.side,
        qty,
        px,
      })
      setError(null)
      setForm({ instrument_id: null, side: 'Buy', qty: '', px: '' })
      setShowForm(false)
      await load()
      onRefresh?.()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Unknown error')
    }
  }


  return (
    <Card className="flex h-full flex-col">
      <CardHeader className="flex flex-col gap-2 p-4">
        <div className="flex flex-col gap-1 sm:flex-row sm:items-center sm:justify-between">
          <CardTitle className="text-lg">{id}</CardTitle>
          <div className="flex flex-col items-end gap-1 sm:flex-row sm:items-center sm:gap-2">
            {calc != null && (
              <div className="flex flex-col items-end gap-0.5">
                <div className="flex items-center gap-2 text-xs">
                  <span className="text-muted-foreground">Total Value:</span>
                  <MetricsDisplay value={calc.total_value} currency={calc.currency} variant="total" />
                </div>
                <div className="flex items-center gap-2 text-xs">
                  <span className="text-muted-foreground">Unrealized PnL:</span>
                  <MetricsDisplay value={calc.unrealized_pnl} currency={calc.currency} variant="badge" />
                </div>
                <div className="flex items-center gap-2 text-xs">
                  <span className="text-muted-foreground">Realized PnL:</span>
                  <MetricsDisplay value={calc.realized_pnl} currency={calc.currency} variant="badge" />
                </div>
              </div>
            )}
            <Button
              variant="secondary"
              size="icon"
              onClick={() => setShowForm((s) => !s)}
              aria-label="Add execution"
              className="h-8 w-8"
            >
              +
            </Button>
          </div>
        </div>
        {currencies.length > 0 && (
          <Select value={currency} onValueChange={(v) => setCurrency(v)}>
            <SelectTrigger className="h-8 w-[90px] text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {currencies.map((c) => (
                <SelectItem key={c.code} value={c.code} className="text-xs">
                  {c.code}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </CardHeader>
      <CardContent className="flex-1 space-y-4 p-4 pt-0">
        {loading ? (
          <p className="text-muted-foreground text-sm">Loading...</p>
        ) : error ? (
          <div role="alert" className="flex items-center justify-between rounded-lg border border-destructive/50 bg-destructive/10 px-3 py-2">
            <p className="text-destructive text-sm">{error}</p>
            <Button variant="outline" size="sm" onClick={() => { setError(null); load() }}>
              Retry
            </Button>
          </div>
        ) : (
          <>
            {showForm && (
              <form onSubmit={handleSubmit} className="space-y-3 rounded-lg border p-3">
                <div className="grid gap-3 sm:grid-cols-2">
                  <div className="space-y-1.5">
                    <Label htmlFor="instrument" className="text-xs">Instrument</Label>
                    <Select
                      value={form.instrument_id ? `${form.instrument_id.symbol}-${form.instrument_id.venue}` : ''}
                      onValueChange={(v) => {
                        const inst = instruments.find((i) => `${i.symbol}-${i.venue}` === v) ?? null
                        setForm((f) => ({ ...f, instrument_id: inst }))
                      }}
                      required
                    >
                      <SelectTrigger id="instrument" className="h-8 text-xs">
                        <SelectValue placeholder="Select instrument" />
                      </SelectTrigger>
                      <SelectContent>
                        {instruments.map((i) => (
                          <SelectItem key={`${i.symbol}-${i.venue}`} value={`${i.symbol}-${i.venue}`} className="text-xs">
                            {i.symbol} ({i.venue})
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-1.5">
                    <Label htmlFor="side" className="text-xs">Side</Label>
                    <Select value={form.side} onValueChange={(v) => setForm((f) => ({ ...f, side: v as Side }))}>
                      <SelectTrigger id="side" className="h-8 text-xs">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="Buy" className="text-xs">Buy</SelectItem>
                        <SelectItem value="Sell" className="text-xs">Sell</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>
                <div className="grid gap-3 sm:grid-cols-2">
                  <div className="space-y-1.5">
                    <Label htmlFor="qty" className="text-xs">Qty</Label>
                    <Input
                      id="qty"
                      type="number"
                      step="any"
                      value={form.qty}
                      onChange={(e) => setForm((f) => ({ ...f, qty: e.target.value }))}
                      required
                      className="h-8 text-xs"
                    />
                  </div>
                  <div className="space-y-1.5">
                    <Label htmlFor="px" className="text-xs">Price</Label>
                    <Input
                      id="px"
                      type="number"
                      step="any"
                      value={form.px}
                      onChange={(e) => setForm((f) => ({ ...f, px: e.target.value }))}
                      required
                      className="h-8 text-xs"
                    />
                  </div>
                </div>
                <Button type="submit" size="sm" className="h-8">Add</Button>
              </form>
            )}

            <div className="space-y-1.5">
              <h4 className="text-sm font-semibold">Positions</h4>
              <div className="overflow-x-auto rounded-md border">
                <PositionSnapshotTable
                  positions={positions}
                  emptyMessage="No positions"
                />
              </div>
            </div>

            <div className="space-y-1.5">
              <h4 className="text-sm font-semibold">Closed Positions</h4>
              <div className="overflow-x-auto rounded-md border">
                <PositionSnapshotTable
                  positions={history}
                  emptyMessage="No closed positions"
                />
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  )
}
