use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionId {
    pub instrument_id: InstrumentId,
    pub portfolio_id: String,
}

impl PositionId {
    pub fn new(instrument_id: InstrumentId, portfolio_id: impl Into<String>) -> Self {
        Self {
            instrument_id,
            portfolio_id: portfolio_id.into(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Currency {
    /// Currency code such as USD, DKK
    pub code: String,
}

impl Currency {
    pub fn new(code: impl Into<String>) -> Self {
        Currency { code: code.into() }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy = 1,
    Sell = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub id: String,
    pub instrument_id: InstrumentId,
    pub portfolio_id: String,
    pub side: Side,
    pub qty: f64,
    pub px: f64,
}

impl Execution {
    pub fn new(instrument_id: InstrumentId, portfolio_id: impl Into<String>, px: f64, qty: f64, side: Side) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            instrument_id,
            portfolio_id: portfolio_id.into(),
            side,
            qty,
            px,
        }
    }

    pub fn position_id(&self) -> PositionId {
        PositionId {
            instrument_id: self.instrument_id.clone(),
            portfolio_id: self.portfolio_id.clone(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PnLError {
    MissingMidPrice,
    MissingFxRate,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Venue(pub String);

impl Venue {
    pub fn new(venue: impl Into<String>) -> Self {
        Venue(venue.into())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstrumentId {
    pub symbol: String,
    pub venue: Venue,
}

impl InstrumentId {
    pub fn new(symbol: impl Into<String>, venue: Venue) -> Self {
        Self {
            symbol: symbol.into(),
            venue,
        }
    }
}

pub trait Instrument: Send + Sync {
    fn id(&self) -> InstrumentId;
    fn base_currency(&self) -> Option<Currency>;
    fn quote_currency(&self) -> Currency;

    // Contract multiplier. I.E. ES Mini 1 movement = 50
    fn multiplier(&self) -> f64;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub id: InstrumentId,
    pub quote_currency: Currency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cash {
    pub id: InstrumentId,
    pub base_currency: Currency,
    pub quote_currency: Currency,
}

impl Instrument for Stock {
    fn id(&self) -> InstrumentId {
        self.id.clone()
    }
    fn base_currency(&self) -> Option<Currency> {
        None
    }
    fn quote_currency(&self) -> Currency {
        self.quote_currency.clone()
    }
    fn multiplier(&self) -> f64 {
        1.0
    }
}

impl Instrument for Cash {
    fn id(&self) -> InstrumentId {
        self.id.clone()
    }
    fn base_currency(&self) -> Option<Currency> {
        Some(self.base_currency.clone())
    }
    fn quote_currency(&self) -> Currency {
        self.quote_currency.clone()
    }
    fn multiplier(&self) -> f64 {
        1.0
    }
}
