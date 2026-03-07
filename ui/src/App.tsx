import { useState, useEffect, type SubmitEvent } from 'react'
import { listPortfolios, createPortfolio, listInstruments, listCurrencies, type Instrument, type Currency } from './api'
import { PortfolioCard } from './components/PortfolioCard'
import { Button } from './components/ui/button'
import { Input } from './components/ui/input'
import { Card, CardContent } from './components/ui/card'

export default function App() {
  const [portfolios, setPortfolios] = useState<string[]>([])
  const [instruments, setInstruments] = useState<Instrument[]>([])
  const [currencies, setCurrencies] = useState<Currency[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [showAdd, setShowAdd] = useState(false)
  const [newName, setNewName] = useState('')

  useEffect(() => {
    load()
  }, [])

  async function load() {
    setLoading(true)
    setError(null)
    try {
      const [portfs, insts, currs] = await Promise.all([
        listPortfolios(),
        listInstruments(),
        listCurrencies(),
      ])
      setPortfolios(portfs)
      setInstruments(insts)
      setCurrencies(currs)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Unknown error')
    } finally {
      setLoading(false)
    }
  }

  async function handleCreatePortfolio(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault()
    if (!newName.trim()) return
    try {
      const res = await createPortfolio(newName.trim())
      setError(null)
      setPortfolios((p) => [...p, res.name])
      setNewName('')
      setShowAdd(false)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Unknown error')
    }
  }

  return (
    <main className="min-h-screen w-full bg-background p-6 md:p-8">
      <div className="mx-auto w-full">
        <header className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <h1 className="text-3xl font-bold tracking-tight md:text-4xl">Portfolios</h1>
          <Button
            variant="secondary"
            onClick={() => setShowAdd((s) => !s)}
            aria-label="Add portfolio"
          >
            + Add Portfolio
          </Button>
        </header>

        {error && (
          <div role="alert" className="mb-6 flex items-center justify-between rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3">
            <p className="text-destructive">{error}</p>
            <Button variant="outline" size="sm" onClick={() => { setError(null); load() }}>
              Retry
            </Button>
          </div>
        )}

        {showAdd && (
          <Card className="mb-8">
            <CardContent className="pt-6">
              <form onSubmit={handleCreatePortfolio} className="flex flex-col gap-4 sm:flex-row sm:items-end">
                <div className="flex-1 space-y-2">
                  <label htmlFor="portfolio-name" className="text-base font-medium">
                    Name
                  </label>
                  <Input
                    id="portfolio-name"
                    value={newName}
                    onChange={(e) => setNewName(e.target.value)}
                    placeholder="Portfolio name"
                    required
                    className="w-full"
                  />
                </div>
                <Button type="submit">Create</Button>
              </form>
            </CardContent>
          </Card>
        )}

        {loading ? (
          <p className="text-xl text-muted-foreground">Loading...</p>
        ) : (
          <div className="grid gap-8 sm:grid-cols-1 lg:grid-cols-2">
            {portfolios.map((id) => (
              <PortfolioCard
                key={id}
                id={id}
                instruments={instruments}
                currencies={currencies}
                onRefresh={load}
              />
            ))}
          </div>
        )}
      </div>
    </main>
  )
}
