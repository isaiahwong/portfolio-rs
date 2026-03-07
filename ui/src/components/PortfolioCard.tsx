import { useState, useEffect, type SubmitEvent } from 'react'
import {
  getPositions,
  getHistory,
  addExecution,
  getPnl,
  type Instrument,
  type PositionSnapshot,
  type Currency,
} from '../api'
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
import { PnlDisplay } from '@/components/PnlDisplay'
import { PositionSnapshotTable } from '@/components/PositionSnapshotTable'

interface PortfolioCardProps {
  id: string
  instruments: Instrument[]
  currencies: Currency[]
  onRefresh?: () => void
}

export function PortfolioCard({ id, instruments, currencies, onRefresh }: PortfolioCardProps) {
  const [positions, setPositions] = useState<PositionSnapshot[]>([])
  const [history, setHistory] = useState<PositionSnapshot[]>([])
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const [expandedHistoryId, setExpandedHistoryId] = useState<string | null>(null)
  const [pnl, setPnl] = useState<{ unrealized_pnl: number; currency: string } | null>(null)
  const [currency, setCurrency] = useState<string>('USD')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [form, setForm] = useState<{
    instrument_id: Instrument | null
    side: string
    qty: string
    px: string
  }>({ instrument_id: null, side: 'Buy', qty: '', px: '' })

  useEffect(() => {
    load()
  }, [id, currency])

  async function load() {
    setLoading(true)
    setError(null)
    try {
      const [positionsRes, historyRes, pnlRes] = await Promise.all([
        getPositions(id),
        getHistory(id),
        getPnl(id, currency).catch(() => null),
      ])
      setPositions(positionsRes)
      setHistory(historyRes)
      setPnl(pnlRes)
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
        side: form.side as 'Buy' | 'Sell',
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

  const realizedPnl =
    positions.reduce((s, p) => s + p.realized_pnl, 0) + history.reduce((s, p) => s + p.realized_pnl, 0)

  return (
    <Card className="flex h-full flex-col">
      <CardHeader className="flex flex-col gap-2">
        <div className="flex flex-col gap-1 sm:flex-row sm:items-center sm:justify-between">
          <CardTitle className="text-xl">{id}</CardTitle>
          <div className="flex flex-col items-end gap-1 sm:flex-row sm:items-center sm:gap-2">
            {pnl != null && (
            <div className="flex flex-col items-end gap-1">
              <div className="flex items-center gap-2 text-sm">
                <span className="text-muted-foreground">Unrealized PnL:</span>
                <PnlDisplay value={pnl.unrealized_pnl} currency={pnl.currency} variant="badge" />
              </div>
              <div className="flex items-center gap-2 text-sm">
                <span className="text-muted-foreground">Realized PnL:</span>
                <PnlDisplay value={realizedPnl} currency={pnl.currency} variant="badge" />
              </div>
            </div>
          )}
          <Button
            variant="secondary"
            size="icon"
            onClick={() => setShowForm((s) => !s)}
            aria-label="Add execution"
          >
            +
          </Button>
          </div>
        </div>
        {currencies.length > 0 && (
          <Select value={currency} onValueChange={(v) => setCurrency(v)}>
            <SelectTrigger className="w-[100px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {currencies.map((c) => (
                <SelectItem key={c.code} value={c.code}>
                  {c.code}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </CardHeader>
      <CardContent className="flex-1 space-y-4">
        {loading ? (
          <p className="text-muted-foreground">Loading...</p>
        ) : error ? (
          <div role="alert" className="flex items-center justify-between rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3">
            <p className="text-destructive">{error}</p>
            <Button variant="outline" size="sm" onClick={() => { setError(null); load() }}>
              Retry
            </Button>
          </div>
        ) : (
          <>
            {showForm && (
              <form onSubmit={handleSubmit} className="space-y-4 rounded-lg border p-4">
                <div className="grid gap-4 sm:grid-cols-2">
                  <div className="space-y-2">
                    <Label htmlFor="instrument" className="text-base">Instrument</Label>
                    <Select
                      value={form.instrument_id ? `${form.instrument_id.symbol}-${form.instrument_id.venue}` : ''}
                      onValueChange={(v) => {
                        const inst = instruments.find((i) => `${i.symbol}-${i.venue}` === v)
                        setForm((f) => ({ ...f, instrument_id: inst ?? null }))
                      }}
                      required
                    >
                      <SelectTrigger id="instrument">
                        <SelectValue placeholder="Select instrument" />
                      </SelectTrigger>
                      <SelectContent>
                        {instruments.map((i) => (
                          <SelectItem key={`${i.symbol}-${i.venue}`} value={`${i.symbol}-${i.venue}`}>
                            {i.symbol} ({i.venue})
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="side" className="text-base">Side</Label>
                    <Select value={form.side} onValueChange={(v) => setForm((f) => ({ ...f, side: v }))}>
                      <SelectTrigger id="side">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="Buy">Buy</SelectItem>
                        <SelectItem value="Sell">Sell</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>
                <div className="grid gap-4 sm:grid-cols-2">
                  <div className="space-y-2">
                    <Label htmlFor="qty" className="text-base">Qty</Label>
                    <Input
                      id="qty"
                      type="number"
                      step="any"
                      value={form.qty}
                      onChange={(e) => setForm((f) => ({ ...f, qty: e.target.value }))}
                      required
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="px" className="text-base">Price</Label>
                    <Input
                      id="px"
                      type="number"
                      step="any"
                      value={form.px}
                      onChange={(e) => setForm((f) => ({ ...f, px: e.target.value }))}
                      required
                    />
                  </div>
                </div>
                <Button type="submit">Add</Button>
              </form>
            )}

            <div className="space-y-2">
              <h4 className="text-lg font-semibold">Positions</h4>
              <div className="overflow-x-auto rounded-md border">
                <PositionSnapshotTable
                  positions={positions}
                  expandedId={expandedId}
                  onExpand={setExpandedId}
                  emptyMessage="No positions"
                  currency={pnl?.currency}
                />
              </div>
            </div>

            <div className="space-y-2">
              <h4 className="text-lg font-semibold">Closed Positions</h4>
              <div className="overflow-x-auto rounded-md border">
                <PositionSnapshotTable
                  positions={history}
                  expandedId={expandedHistoryId}
                  onExpand={setExpandedHistoryId}
                  emptyMessage="No closed positions"
                  currency={pnl?.currency}
                />
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  )
}
