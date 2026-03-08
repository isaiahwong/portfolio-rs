import type {
  PositionSnapshot,
  MarketData,
  PortfolioCalc,
  ExecutionRequest,
} from './types'

async function fetchAPI<T>(slug: string, opts: RequestInit = {}): Promise<T> {
  const res = await fetch(slug, {
    ...opts,
    headers: { 'Content-Type': 'application/json', ...opts.headers },
  })
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error((err as { error?: string }).error ?? res.statusText)
  }
  return res.json()
}

export function listPortfolios(): Promise<string[]> {
  return fetchAPI('/portfolios')
}

export function createPortfolio(name: string): Promise<{ name: string }> {
  return fetchAPI('/portfolios', {
    method: 'POST',
    body: JSON.stringify({ name }),
  })
}

export function getPositions(portfolioId: string): Promise<PositionSnapshot[]> {
  return fetchAPI(`/portfolios/${encodeURIComponent(portfolioId)}/positions`)
}

export function getHistory(portfolioId: string): Promise<PositionSnapshot[]> {
  return fetchAPI(`/portfolios/${encodeURIComponent(portfolioId)}/history`)
}

export function addExecution(
  portfolioId: string,
  body: ExecutionRequest
): Promise<unknown> {
  return fetchAPI(`/portfolios/${encodeURIComponent(portfolioId)}/executions`, {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

export function getPortfolioCalc(
  portfolioId: string,
  currency = 'USD'
): Promise<PortfolioCalc> {
  return fetchAPI(`/portfolios/${encodeURIComponent(portfolioId)}/calc?currency=${currency}`)
}

export function getMarketData(): Promise<MarketData> {
  return fetchAPI('/marketdata')
}
