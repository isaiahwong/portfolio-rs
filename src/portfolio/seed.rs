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

pub fn toyota_id() -> InstrumentId {
    InstrumentId::new("7203", Venue::new("TSE"))
}

pub fn asml_id() -> InstrumentId {
    InstrumentId::new("ASML", Venue::new("EURONEXT"))
}

pub fn jpy_id() -> InstrumentId {
    InstrumentId::new("JPY", Venue::new("FX"))
}

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
            id: toyota_id(),
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

pub struct Providers {
    pub currencies: Vec<Currency>,
    instruments: HashMap<InstrumentId, Arc<dyn Instrument>>,
    fx_rates: Mutex<HashMap<(Currency, Currency), f64>>,
    prices: Mutex<HashMap<InstrumentId, f64>>,
}

impl Providers {
    pub fn instrument_ids(&self) -> Vec<InstrumentId> {
        self.instruments.keys().cloned().collect()
    }
}

impl MarketData for Providers {
    fn get_instrument(&self, id: &InstrumentId) -> Option<Arc<dyn Instrument>> {
        self.instruments.get(id).cloned()
    }

    fn get_rate(&self, from: &Currency, to: &Currency) -> Option<f64> {
        self.fx_rates.lock().unwrap().get(&(from.clone(), to.clone())).copied()
    }

    fn get_price(&self, instrument_id: &InstrumentId) -> Option<f64> {
        self.prices.lock().unwrap().get(instrument_id).copied()
    }
}

pub struct AppState {
    pub providers: Arc<Providers>,
    pub portfolios: RwLock<HashMap<String, Arc<Mutex<Portfolio>>>>,
}

pub fn load() -> AppState {
    let usd = Currency::new("USD");
    let eur = Currency::new("EUR");
    let jpy = Currency::new("JPY");
    let sgd = Currency::new("SGD");

    let instruments = instruments();

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
        (aapl_id(), 175.0),
        (toyota_id(), 2500.0),
        (asml_id(), 850.0),
        (usd_id(), 1.0),
        (eur_id(), 1.08),
        (jpy_id(), 0.0067),
    ];
    let prices = Mutex::new(prices_arr.into_iter().collect());

    let provider = Arc::new(Providers {
        instruments,
        currencies: vec![usd, eur, sgd, jpy],
        fx_rates,
        prices,
    });

    let mut alpha = Portfolio::new(provider.clone());
    alpha.on_exec(Execution::new(aapl_id(), "alpha", 170.0, 100.0, Side::Buy));

    let mut hedge = Portfolio::new(provider.clone());
    hedge.on_exec(Execution::new(toyota_id(), "hedge", 100.0, 50.0, Side::Sell));

    let mut portfolios = HashMap::new();
    portfolios.insert("alpha".into(), Arc::new(Mutex::new(alpha)));
    portfolios.insert("hedge".into(), Arc::new(Mutex::new(hedge)));

    println!("Loaded {} portfolios", portfolios.len());
    AppState {
        providers: provider,
        portfolios: RwLock::new(portfolios),
    }
}
