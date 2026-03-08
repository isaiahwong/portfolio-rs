import {
  createContext,
  useContext,
  useMemo,
  type ReactNode,
} from 'react'
import type { InstrumentId, MarketData, Currency } from '../types'

function instrumentKey(inst: InstrumentId): string {
  return `${inst.symbol}-${inst.venue}`
}

interface MarketDataContextValue {
  currencies: Currency[]
  instruments: InstrumentId[]
  midPrices: Map<string, number>
  getMidPrice: (inst: InstrumentId) => number | undefined
}

const MarketDataContext = createContext<MarketDataContextValue | null>(null)

interface MarketDataProviderProps {
  marketData: MarketData | null
  children: ReactNode
}

export function MarketDataProvider({
  marketData,
  children,
}: MarketDataProviderProps) {
  const value = useMemo(() => {
    if (!marketData) {
      return {
        currencies: [],
        instruments: [],
        midPrices: new Map<string, number>(),
        getMidPrice: () => undefined,
      }
    }

    // Cache mid prices
    const midPrices = new Map<string, number>()
    for (const [inst, price] of marketData.mid_prices) {
      midPrices.set(instrumentKey(inst), price)
    }

    return {
      currencies: marketData.currencies,
      instruments: marketData.instruments,
      midPrices,
      getMidPrice: (inst: InstrumentId) => midPrices.get(instrumentKey(inst)),
    }
  }, [marketData])

  return (
    <MarketDataContext.Provider value={value}>
      {children}
    </MarketDataContext.Provider>
  )
}

export function useMarketData() {
  const ctx = useContext(MarketDataContext)
  if (!ctx) {
    throw new Error('useMarketData must be used within MarketDataProvider')
  }
  return ctx
}
