use serde::{Deserialize, Serialize};

/// Identifier for a position within a portfolio.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionId {
    pub instrument_id: InstrumentId,
    pub portfolio_id: String,
}

impl PositionId {
    #[allow(dead_code)]
    pub fn new(instrument_id: InstrumentId, portfolio_id: impl Into<String>) -> Self {
        Self {
            instrument_id,
            portfolio_id: portfolio_id.into(),
        }
    }
}

/// Currency with metadata such as code.
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

/// Side of an execution/order.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy = 1,
    Sell = 2,
}

/// Record of a completed trade execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    /// Unique identifier for the execution.
    pub id: String,
    /// Identifier for the traded instrument.
    pub instrument_id: InstrumentId,
    /// The portfolio associated with the execution.
    pub portfolio_id: String,
    /// Exec side.
    pub side: Side,
    /// Quantity executed.
    pub qty: f64,
    /// Price per unit.
    pub px: f64,
}

impl Execution {
    pub fn new(instrument_id: InstrumentId, portfolio_id: impl Into<String>, px: f64, qty: f64, side: Side) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(), // FIXME: used for poc only.
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

/// Represents a trading venue or exchange.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Venue(pub String);

impl Venue {
    pub fn new(venue: impl Into<String>) -> Self {
        Venue(venue.into())
    }
}

/// Unique identifier for a financial instrument.
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

/// Trait for all financial instruments.
pub trait Instrument: Send + Sync {
    /// Returns the unique identifier for the instrument.
    fn id(&self) -> InstrumentId;

    /// Returns the base currency of the instrument, if applicable. I.E. FX Pairs or crypto.
    fn base_currency(&self) -> Option<Currency>;

    /// Returns the currency in which the instrument is quoted.
    fn quote_currency(&self) -> Currency;

    /// Returns the contract multiplier. I.E. CME ES mini has a multiplier of 50.
    fn multiplier(&self) -> f64;
}

/// Represents an equity instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub id: InstrumentId,
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

/// Represents a cash or FX instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cash {
    pub id: InstrumentId,
    pub base_currency: Currency,
    pub quote_currency: Currency,
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

/// Portfolio errors.
#[derive(Debug, PartialEq)]
pub enum Error {
    MidPrice,
    FxRate,
    InstrumentNotFound,
}
