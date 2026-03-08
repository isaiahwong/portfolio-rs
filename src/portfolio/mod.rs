use std::{collections::HashMap, sync::Arc};

use crate::portfolio::{
    position::{Position, PositionSnapshot},
    types::{Currency, Error, Execution, Instrument, InstrumentId, PositionId},
};

pub mod position;
pub mod seed;
pub mod types;

/// Trait for providing market data such as instrument, FX, and mid prices.
pub trait MarketData: Send + Sync {
    fn get_instrument(&self, id: &InstrumentId) -> Option<Arc<dyn Instrument>>;
    fn get_rate(&self, from: &Currency, to: &Currency) -> Option<f64>;
    fn get_price(&self, instrument_id: &InstrumentId) -> Option<f64>;
}

/// Manages a collection of positions.
pub struct Portfolio {
    positions: HashMap<PositionId, Position>,
    history: HashMap<PositionId, Vec<PositionSnapshot>>,
    marketdata: Arc<dyn MarketData>,
}

impl Portfolio {
    /// Creates a new empty portfolio.
    pub fn new(marketdata: Arc<dyn MarketData>) -> Self {
        Self {
            positions: HashMap::<PositionId, Position>::new(),
            history: HashMap::<PositionId, Vec<PositionSnapshot>>::new(),
            marketdata,
        }
    }

    /// Processes an execution, creating or updating positions.
    pub fn on_exec(&mut self, execution: Execution) -> Result<(), Error> {
        let position_id = execution.position_id();

        let instrument = self
            .marketdata
            .get_instrument(&execution.instrument_id)
            .ok_or(Error::InstrumentNotFound)?;

        let pos_opt = self.positions.remove(&position_id);

        let updated_pos = match pos_opt {
            // Open new position
            None => Position::new(instrument.as_ref(), execution),

            // Update existing position
            Some(mut pos) => {
                // position flip sides
                if Self::will_pos_flip(&pos, &execution) {
                    self.flip_position(instrument.as_ref(), &position_id, pos, execution)
                } else {
                    pos.apply(execution);
                    pos
                }
            }
        };

        match &updated_pos.status {
            position::PositionStatus::Open => {
                let _ = self.positions.insert(position_id, updated_pos);
            }
            position::PositionStatus::Closed => {
                self.history.entry(position_id.clone()).or_default().push((&updated_pos).into());
            }
        };

        Ok(())
    }

    /// Closes the current position and opens a new one on the opposite direction.
    fn flip_position(
        &mut self,
        instrument: &dyn Instrument,
        position_id: &PositionId,
        mut pos: Position,
        execution: Execution,
    ) -> Position {
        let mut close_exec = execution.clone();
        close_exec.qty = pos.qty;

        let mut open_exec = execution;
        open_exec.qty -= pos.qty;

        // Close previous direction position and snapshot
        pos.apply(close_exec);
        self.history.entry(position_id.clone()).or_default().push((&pos).into());

        // Create position with new side
        Position::new(instrument, open_exec)
    }

    fn will_pos_flip(pos: &Position, execution: &Execution) -> bool {
        pos.side != execution.side && execution.qty > pos.qty
    }

    /// Calculates the total unrealized PnL across all open positions in the target currency.
    pub fn calc_total_unrealized_pnl(&self, target: Currency) -> Result<f64, Error> {
        let mut total = 0.0;

        for pos in self.positions.values() {
            let inst_id = &pos.id.instrument_id;
            let mid_px = self.marketdata.get_price(inst_id).ok_or(Error::MidPrice)?;
            let u_pnl = pos.unrealized_pnl(mid_px);

            if pos.quote_currency == target {
                total += u_pnl;
            } else {
                let fx = self.marketdata.get_rate(&pos.quote_currency, &target).ok_or(Error::FxRate)?;
                total += fx * u_pnl;
            }
        }

        Ok(total)
    }

    /// Calculates the total realized PnL from both open and closed positions in the target currency.
    pub fn calc_total_realized_pnl(&self, target: Currency) -> Result<f64, Error> {
        let mut total = 0.0;

        // Calc realized pnl for active positions
        for pos in self.positions.values() {
            let r_pnl = pos.realized_pnl;

            if pos.quote_currency == target {
                total += r_pnl;
            } else {
                let fx = self.marketdata.get_rate(&pos.quote_currency, &target).ok_or(Error::FxRate)?;
                total += fx * r_pnl;
            }
        }

        // Calc realized pnl for closed positions
        for snapshots in self.history.values() {
            for snapshot in snapshots {
                let r_pnl = snapshot.realized_pnl;

                if snapshot.quote_currency == target {
                    total += r_pnl;
                } else {
                    let fx = self.marketdata.get_rate(&snapshot.quote_currency, &target).ok_or(Error::FxRate)?;
                    total += fx * r_pnl;
                }
            }
        }

        Ok(total)
    }

    /// Calculates the total market value of all open positions in the target currency.
    pub fn calc_total_value(&self, target: Currency) -> Result<f64, Error> {
        let mut total = 0.0;

        // Sum notional for open positions
        for pos in self.positions.values() {
            let inst_id = &pos.id.instrument_id;
            let mid_px = self.marketdata.get_price(inst_id).ok_or(Error::MidPrice)?;

            let value_quote = pos.notional(mid_px);

            if pos.quote_currency == target {
                total += value_quote;
            } else {
                let fx = self.marketdata.get_rate(&pos.quote_currency, &target).ok_or(Error::FxRate)?;
                total += fx * value_quote;
            }
        }

        Ok(total)
    }

    /// Returns snapshots of all open positions.
    pub fn positions(&self) -> Vec<PositionSnapshot> {
        self.positions.values().map(|pos| pos.into()).collect()
    }

    /// Returns snapshots of all closed position.
    pub fn history(&self) -> Vec<PositionSnapshot> {
        self.history.values().flat_map(|snapshots| snapshots.clone()).collect()
    }

    #[allow(dead_code)]
    pub fn position(&self, id: &PositionId) -> Option<&Position> {
        self.positions.get(id)
    }

    #[allow(dead_code)]
    pub fn position_history(&self, id: &PositionId) -> Option<&Vec<PositionSnapshot>> {
        self.history.get(id)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use super::position::PositionSide;
    use super::seed::{aapl_id, eur_id, tyo_id, usd_id};
    use super::types::{Currency, Execution, Instrument, InstrumentId, PositionId, Side};
    use super::*;

    /// Portfolio Id
    pub const PID: &str = "test-portfolio";

    #[test]
    fn test_multiple_assets() {
        let (mut portfolio, _) = setup();

        portfolio.on_exec(Execution::new(aapl_id(), PID, 2.00, 100.0, Side::Buy)).unwrap();
        portfolio.on_exec(Execution::new(aapl_id(), PID, 3.50, 200.0, Side::Buy)).unwrap();
        portfolio.on_exec(Execution::new(tyo_id(), PID, 5.50, 50.0, Side::Buy)).unwrap();
        portfolio.on_exec(Execution::new(tyo_id(), PID, 7.75, 25.0, Side::Buy)).unwrap();
        portfolio.on_exec(Execution::new(eur_id(), PID, 1.0, 1000.0, Side::Buy)).unwrap();
        portfolio.on_exec(Execution::new(eur_id(), PID, 1.0, 200.0, Side::Buy)).unwrap();
        portfolio.on_exec(Execution::new(usd_id(), PID, 1.0, 250.0, Side::Buy)).unwrap();

        let aapl_pos = portfolio.position(&PositionId::new(aapl_id(), PID)).unwrap();
        assert_eq!(aapl_pos.qty, 300.0);
        assert_eq!(aapl_pos.avg_px, 3.00);

        let toyota_pos = portfolio.position(&PositionId::new(tyo_id(), PID)).unwrap();
        assert_eq!(toyota_pos.qty, 75.0);
        assert_eq!(toyota_pos.avg_px, 6.25);

        let eur_pos = portfolio.position(&PositionId::new(eur_id(), PID)).unwrap();
        assert_eq!(eur_pos.qty, 1200.0);

        let usd_pos = portfolio.position(&PositionId::new(usd_id(), PID)).unwrap();
        assert_eq!(usd_pos.qty, 250.0);
    }

    #[test]
    fn test_flip_position() {
        let (mut portfolio, _) = setup();

        // Long aapl 100 long @ 100
        portfolio.on_exec(Execution::new(aapl_id(), PID, 100.0, 100.0, Side::Buy)).unwrap();

        // Flip sell aapl 150 @ 95 -> closes 100 long, opens 50 short @ 95
        // PnL: (95 - 100) * 100 = -500
        portfolio.on_exec(Execution::new(aapl_id(), PID, 95.0, 150.0, Side::Sell)).unwrap();

        let aapl_pos = portfolio.position(&PositionId::new(aapl_id(), PID)).unwrap();
        assert_eq!(aapl_pos.qty, 50.0);
        assert_eq!(aapl_pos.avg_px, 95.0);
        assert_eq!(aapl_pos.position_side, PositionSide::Short);
        assert_eq!(aapl_pos.realized_pnl, 0.0);

        // PnL (95 - 100) * 100 = -500
        let history = portfolio.position_history(&PositionId::new(aapl_id(), PID)).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].realized_pnl, -500.0);
    }

    #[test]
    fn test_unrealized_pnl() {
        let (mut portfolio, marketdata) = setup();
        let usd = Currency { code: "USD".into() };
        let jpy = Currency { code: "JPY".into() };

        // Long aapl 100 shares @ 100 (Quote: USD)
        portfolio.on_exec(Execution::new(aapl_id(), PID, 100.0, 100.0, Side::Buy)).unwrap();
        // Long toyota 100 shares @ 50 (Quote: JPY)
        portfolio.on_exec(Execution::new(tyo_id(), PID, 50.0, 100.0, Side::Buy)).unwrap();

        // mid change for aapl -> 110 USD
        marketdata.set_price(aapl_id(), 110.0);
        // mid change for toyota -> 60 JPY
        marketdata.set_price(tyo_id(), 60.0);

        // Valuation in USD - 1 JPY = 0.01 USD
        marketdata.set_rate(jpy.clone(), usd.clone(), 0.01);

        // PnL (USD) = 1000 USD + (1000 JPY * 0.01) = 1010 USD
        let total_pnl_usd = portfolio.calc_total_unrealized_pnl(usd.clone()).unwrap();
        assert_eq!(total_pnl_usd, 1010.0);

        // Valuation in JPY - 1 USD = 100 JPY
        marketdata.set_rate(usd.clone(), jpy.clone(), 100.0);

        // PnL (JPY) = (1000 USD * 100) + 1000 JPY = 101000 JPY
        let total_pnl_jpy = portfolio.calc_total_unrealized_pnl(jpy).unwrap();
        assert_eq!(total_pnl_jpy, 101000.0);
    }

    /// Mock implementation of MarketData used for unit testing.
    pub struct TestMarketData {
        pub instruments: HashMap<InstrumentId, Arc<dyn Instrument>>,
        pub fx_rates: Mutex<HashMap<(Currency, Currency), f64>>,
        pub prices: Mutex<HashMap<InstrumentId, f64>>,
    }

    impl TestMarketData {
        pub fn new() -> Self {
            Self {
                instruments: super::seed::instruments(),
                fx_rates: Mutex::new(HashMap::new()),
                prices: Mutex::new(HashMap::new()),
            }
        }

        pub fn set_price(&self, id: InstrumentId, px: f64) {
            self.prices.lock().unwrap().insert(id, px);
        }

        pub fn set_rate(&self, from: Currency, to: Currency, rate: f64) {
            self.fx_rates.lock().unwrap().insert((from, to), rate);
        }
    }

    impl MarketData for TestMarketData {
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

    pub fn setup() -> (Portfolio, Arc<TestMarketData>) {
        let marketdata = Arc::new(TestMarketData::new());
        let portfolio = Portfolio::new(marketdata.clone());
        (portfolio, marketdata)
    }
}
