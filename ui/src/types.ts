export interface InstrumentId {
  symbol: string
  venue: string
}

export interface Currency {
  code: string
}

export type Side = 'Buy' | 'Sell'

export interface Execution {
  id: string
  instrument_id: InstrumentId
  portfolio_id: string
  side: Side
  qty: number
  px: number
}

export interface PositionId {
  instrument_id: InstrumentId
  portfolio_id: string
}

export interface PositionSnapshot {
  id: string
  position_id: PositionId
  executions: Execution[]
  position_side: 'Flat' | 'Long' | 'Short'
  qty: number
  base_currency: Currency | null
  quote_currency: Currency
  avg_px: number
  realized_pnl: number
}

export interface MarketData {
  currencies: Currency[]
  instruments: InstrumentId[]
  mid_prices: [InstrumentId, number][]
}

export interface PortfolioCalc {
  currency: string
  unrealized_pnl: number
  realized_pnl: number
  total_value: number
}

export interface ExecutionRequest {
  instrument_id: InstrumentId
  side: Side
  qty: number
  px: number
}
