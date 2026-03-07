const BASE = ''

async function fetchJson<T>(slug: string, opts: RequestInit = {}): Promise<T> {
  const res = await fetch(`${BASE}${slug}`, {
    ...opts,
    headers: { 'Content-Type': 'application/json', ...opts.headers },
  })
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error((err as { error?: string }).error ?? res.statusText)
  }
  return res.json()
}

export interface InstrumentId {
  symbol: string
  venue: string
}

export type Instrument = InstrumentId

export interface Execution {
  id: string
  instrument_id: InstrumentId
  portfolio_id: string
  side: 'Buy' | 'Sell'
  qty: number
  px: number
}

export interface PositionSnapshot {
  id: string
  position_id: { instrument_id: InstrumentId; portfolio_id: string }
  executions: Execution[]
  position_side: 'Flat' | 'Long' | 'Short'
  qty: number
  base_currency: { code: string } | null
  quote_currency: { code: string }
  avg_px: number
  realized_pnl: number
}

export interface Pnl {
  unrealized_pnl: number
  currency: string
}

export interface CreatePortfolioResponse {
  name: string
}

export function listPortfolios(): Promise<string[]> {
  return fetchJson('/portfolios')
}

export function createPortfolio(name: string): Promise<CreatePortfolioResponse> {
  return fetchJson('/portfolios', {
    method: 'POST',
    body: JSON.stringify({ name: name || undefined }),
  })
}

export function getPositions(portfolioId: string): Promise<PositionSnapshot[]> {
  return fetchJson(`/portfolios/${encodeURIComponent(portfolioId)}/positions`)
}

export function getHistory(portfolioId: string): Promise<PositionSnapshot[]> {
  return fetchJson(`/portfolios/${encodeURIComponent(portfolioId)}/history`)
}

export function getExecutions(portfolioId: string): Promise<Execution[]> {
  return fetchJson(`/portfolios/${encodeURIComponent(portfolioId)}/executions`)
}

export function addExecution(
  portfolioId: string,
  body: { instrument_id: InstrumentId; side: 'Buy' | 'Sell'; qty: number; px: number }
): Promise<unknown> {
  return fetchJson(`/portfolios/${encodeURIComponent(portfolioId)}/executions`, {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export function getPnl(portfolioId: string, currency = 'USD'): Promise<Pnl> {
  return fetchJson(`/portfolios/${encodeURIComponent(portfolioId)}/pnl?currency=${currency}`)
}

export function listInstruments(): Promise<Instrument[]> {
  return fetchJson('/instruments')
}

export interface Currency {
  code: string
}

export function listCurrencies(): Promise<Currency[]> {
  return fetchJson('/currencies')
}
