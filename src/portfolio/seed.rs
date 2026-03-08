use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::portfolio::{
    MarketData, Portfolio,
    types::{Cash, Currency, Execution, Instrument, InstrumentId, Side, Stock, Venue},
};

pub fn eur_id() -> InstrumentId {
    InstrumentId::new("EUR", Venue::new("FX"))
}

pub fn usd_id() -> InstrumentId {
    InstrumentId::new("USD", Venue::new("FX"))
}

pub fn aapl_id() -> InstrumentId {
    InstrumentId::new("AAPL", Venue::new("NASDAQ"))
}

pub fn tyo_id() -> InstrumentId {
    InstrumentId::new("TOYOTA.7203", Venue::new("TSE"))
}

pub fn asml_id() -> InstrumentId {
    InstrumentId::new("ASML", Venue::new("EURONEXT"))
}

pub fn jpy_id() -> InstrumentId {
    InstrumentId::new("JPY", Venue::new("FX"))
}

/// Returns a predefined map of available instruments.
pub fn instruments() -> HashMap<InstrumentId, Arc<dyn Instrument>> {
    let usd = Currency::new("USD");
    let eur = Currency::new("EUR");
    let jpy = Currency::new("JPY");

    vec![
        Arc::new(Stock {
            id: aapl_id(),
            quote_currency: usd.clone(),
        }) as Arc<dyn Instrument>,
        Arc::new(Stock {
            id: tyo_id(),
            quote_currency: jpy.clone(),
        }) as Arc<dyn Instrument>,
        Arc::new(Stock {
            id: asml_id(),
            quote_currency: eur.clone(),
        }) as Arc<dyn Instrument>,
        Arc::new(Cash {
            id: usd_id(),
            base_currency: usd.clone(),
            quote_currency: usd.clone(),
        }) as Arc<dyn Instrument>,
        Arc::new(Cash {
            id: eur_id(),
            base_currency: eur.clone(),
            quote_currency: eur.clone(),
        }) as Arc<dyn Instrument>,
        Arc::new(Cash {
            id: jpy_id(),
            base_currency: jpy.clone(),
            quote_currency: jpy.clone(),
        }) as Arc<dyn Instrument>,
    ]
    .into_iter()
    .map(|i| (i.id().clone(), i))
    .collect()
}

/// In-memory market data.
pub struct InMemMarketdata {
    pub currencies: Vec<Currency>,
    instruments: HashMap<InstrumentId, Arc<dyn Instrument>>,
    fx_rates: Mutex<HashMap<(Currency, Currency), f64>>,
    prices: Mutex<HashMap<InstrumentId, f64>>,
}

impl InMemMarketdata {
    pub fn instrument_ids(&self) -> Vec<InstrumentId> {
        self.instruments.keys().cloned().collect()
    }

    pub fn mid_prices(&self) -> Vec<(InstrumentId, f64)> {
        self.prices.lock().unwrap().iter().map(|(k, &v)| (k.clone(), v)).collect()
    }
}

impl MarketData for InMemMarketdata {
    fn get_instrument(&self, id: &InstrumentId) -> Option<Arc<dyn Instrument>> {
        self.instruments.get(id).cloned()
    }

    fn get_price(&self, instrument_id: &InstrumentId) -> Option<f64> {
        self.prices.lock().unwrap().get(instrument_id).copied()
    }

    fn get_rate(&self, from: &Currency, to: &Currency) -> Option<f64> {
        let rates = self.fx_rates.lock().unwrap();

        if let Some(&rate) = rates.get(&(from.clone(), to.clone())) {
            return Some(rate);
        }

        // Triangulation via USD
        let usd = Currency::new("USD");
        let rate1 = rates.get(&(from.clone(), usd.clone()))?;
        let rate2 = rates.get(&(usd.clone(), to.clone()))?;

        Some(rate1 * rate2)
    }
}

/// AppState containing deps such as inmem marketdata and portfolios.
pub struct AppState {
    pub marketdata: Arc<InMemMarketdata>,
    pub portfolios: RwLock<HashMap<String, Arc<Mutex<Portfolio>>>>,
}

/// Initializes and loads the app state with seed data.
pub fn load_appstate() -> AppState {
    let usd = Currency::new("USD");
    let eur = Currency::new("EUR");
    let jpy = Currency::new("JPY");
    let sgd = Currency::new("SGD");

    let fx_rates_arr = [
        ((eur.clone(), usd.clone()), 1.08),
        ((usd.clone(), eur.clone()), 1.0 / 1.08),
        ((jpy.clone(), usd.clone()), 0.0067),
        ((usd.clone(), jpy.clone()), 1.0 / 0.0067),
        ((sgd.clone(), usd.clone()), 0.78),
        ((usd.clone(), sgd.clone()), 1.0 / 0.78),
        ((usd.clone(), usd.clone()), 1.0),
        ((eur.clone(), eur.clone()), 1.0),
        ((jpy.clone(), jpy.clone()), 1.0),
        ((sgd.clone(), sgd.clone()), 1.0),
    ];
    let fx_rates = Mutex::new(fx_rates_arr.into_iter().collect());

    let prices_arr = [
        (aapl_id(), 257.0),
        (tyo_id(), 3515.0),
        (asml_id(), 1147.0),
        (usd_id(), 1.0),
        (eur_id(), 1.0),
        (jpy_id(), 1.0),
    ];
    let prices = Mutex::new(prices_arr.into_iter().collect());

    let marketdata = Arc::new(InMemMarketdata {
        instruments: instruments(),
        currencies: vec![usd, eur, sgd, jpy],
        fx_rates,
        prices,
    });

    // Create alpha portfolio with initial executions
    let mut alpha = Portfolio::new(marketdata.clone());
    alpha.on_exec(Execution::new(aapl_id(), "alpha", 300.0, 120.0, Side::Sell)).unwrap();
    alpha.on_exec(Execution::new(aapl_id(), "alpha", 260.0, 200.0, Side::Buy)).unwrap();
    alpha.on_exec(Execution::new(aapl_id(), "alpha", 255.0, 10.0, Side::Sell)).unwrap();
    alpha.on_exec(Execution::new(asml_id(), "alpha", 1000.0, 40.0, Side::Sell)).unwrap();
    alpha.on_exec(Execution::new(asml_id(), "alpha", 1200.0, 10.0, Side::Sell)).unwrap();

    // Create hedge portfolio with initial executions
    let mut hedge = Portfolio::new(marketdata.clone());
    hedge.on_exec(Execution::new(tyo_id(), "hedge", 3000.0, 50.0, Side::Sell)).unwrap();
    hedge.on_exec(Execution::new(tyo_id(), "hedge", 2000.0, 100.0, Side::Buy)).unwrap();
    hedge.on_exec(Execution::new(tyo_id(), "hedge", 2200.0, 10.0, Side::Buy)).unwrap();
    hedge.on_exec(Execution::new(asml_id(), "hedge", 1220.0, 100.0, Side::Buy)).unwrap();
    hedge.on_exec(Execution::new(asml_id(), "hedge", 1240.0, 40.0, Side::Sell)).unwrap();
    hedge.on_exec(Execution::new(eur_id(), "hedge", 1.0, 1000.0, Side::Buy)).unwrap();
    hedge.on_exec(Execution::new(usd_id(), "hedge", 1.0, 250.0, Side::Buy)).unwrap();
    hedge.on_exec(Execution::new(usd_id(), "hedge", 1.0, 250.0, Side::Buy)).unwrap();

    let mut portfolios = HashMap::new();
    portfolios.insert("alpha".into(), Arc::new(Mutex::new(alpha)));
    portfolios.insert("hedge".into(), Arc::new(Mutex::new(hedge)));

    AppState {
        marketdata,
        portfolios: RwLock::new(portfolios),
    }
}
