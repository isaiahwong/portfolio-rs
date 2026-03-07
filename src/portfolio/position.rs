use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::{Currency, Execution, Instrument, PositionId, Side};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionSide {
    Flat,
    Long,
    Short,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionStatus {
    Open,
    Closed,
}

pub struct Position {
    /// The id that pairs instrument_id and portfolio_id.
    pub id: PositionId,
    /// The executions related to the position.
    pub executions: Vec<Execution>,
    /// The direction of the position.
    pub position_side: PositionSide,
    /// The starting side of the position. Used to check for flip side.
    pub side: Side,
    /// The signed quantity of the position. -ve for short, +ve for long.
    pub signed_qty: f64,
    /// The abs quantity of the position.
    pub qty: f64,
    /// The contract multiplier. I.E. ES Mini has $50 to 1 point
    pub multiplier: f64,
    /// The base currency for the position.
    pub base_currency: Option<Currency>,
    /// The quote currency for the position.
    pub quote_currency: Currency,
    /// The average weighted price of position
    pub avg_px: f64,
    /// The realized pnl for position.
    pub realized_pnl: f64,
    /// The status for the position.
    pub status: PositionStatus,
}

impl Position {
    pub fn new(instrument: &dyn Instrument, execution: Execution) -> Self {
        let mut pos = Self {
            id: PositionId {
                instrument_id: instrument.id().clone(),
                portfolio_id: execution.portfolio_id.clone(),
            },
            executions: Vec::<Execution>::new(),
            side: execution.side,
            position_side: PositionSide::Flat,
            signed_qty: 0.0,
            qty: 0.0,
            base_currency: instrument.base_currency(),
            quote_currency: instrument.quote_currency(),
            multiplier: instrument.multiplier(),
            avg_px: 0.0,
            realized_pnl: 0.0,
            status: PositionStatus::Open,
        };

        pos.apply(execution);
        pos
    }

    pub fn apply(&mut self, execution: Execution) {
        if self.position_side == PositionSide::Flat {
            self.avg_px = execution.px;
            self.realized_pnl = 0.0
        }

        // Apply execution by side
        match &execution.side {
            Side::Buy => self.apply_buy(&execution),
            Side::Sell => self.apply_sell(&execution),
        }

        self.qty = self.signed_qty.abs();

        match signed(self.signed_qty) {
            Signed::Zero => {
                self.position_side = PositionSide::Flat;
                self.status = PositionStatus::Closed;
            }

            Signed::Positive => {
                self.position_side = PositionSide::Long;
            }

            Signed::Negative => {
                self.position_side = PositionSide::Short;
            }
        }

        self.executions.push(execution);
    }

    fn apply_buy(&mut self, execution: &Execution) {
        let px = execution.px;
        let qty = execution.qty;

        match self.position_side {
            // Add long position
            PositionSide::Long => self.avg_px = self.calc_avg_px(self.avg_px, self.qty, px, qty),

            // Reduce short
            PositionSide::Short => self.realized_pnl += self.calc_pnl(self.avg_px, self.qty, px, qty),

            PositionSide::Flat => {}
        }

        self.signed_qty += qty;
    }

    fn apply_sell(&mut self, execution: &Execution) {
        let px = execution.px;
        let qty = execution.qty;

        match self.position_side {
            // Add short position
            PositionSide::Short => self.avg_px = self.calc_avg_px(self.avg_px, self.qty, px, qty),

            // Reduce long
            PositionSide::Long => self.realized_pnl += self.calc_pnl(self.avg_px, self.qty, px, qty),

            PositionSide::Flat => {}
        }

        self.signed_qty -= qty;
    }

    pub fn unrealized_pnl(&self, market_px: f64) -> f64 {
        if self.position_side == PositionSide::Flat {
            return 0.0;
        }

        self.calc_pnl(self.avg_px, self.qty, market_px, self.qty)
    }

    fn calc_avg_px(&self, cur_avg_px: f64, cur_qty: f64, px: f64, qty: f64) -> f64 {
        let cur_cost = cur_avg_px * cur_qty;
        let new_cost = px * qty;
        let total_qty = cur_qty + qty;

        // TODO: check zero div
        (cur_cost + new_cost) / total_qty
    }

    fn calc_pnl(&self, cur_avg_px: f64, cur_qty: f64, close_px: f64, close_qty: f64) -> f64 {
        let qty = cur_qty.min(close_qty);
        let points = match self.position_side {
            PositionSide::Long => close_px - cur_avg_px,
            PositionSide::Short => cur_avg_px - close_px,
            PositionSide::Flat => 0.0,
        };

        qty * points * self.multiplier
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSnapshot {
    pub id: String,
    pub position_id: PositionId,
    pub executions: Vec<Execution>,
    pub position_side: PositionSide,
    pub qty: f64,
    pub base_currency: Option<Currency>,
    pub quote_currency: Currency,
    pub avg_px: f64,
    pub realized_pnl: f64,
}

impl From<&Position> for PositionSnapshot {
    fn from(pos: &Position) -> Self {
        PositionSnapshot {
            id: Uuid::now_v7().into(),
            position_id: pos.id.clone(),
            executions: pos.executions.clone(),
            position_side: pos.position_side,
            qty: pos.qty,
            base_currency: pos.base_currency.clone(),
            quote_currency: pos.quote_currency.clone(),
            avg_px: pos.avg_px,
            realized_pnl: pos.realized_pnl,
        }
    }
}

const EPSILON: f64 = 0.0001;
enum Signed {
    Positive,
    Negative,
    Zero,
}

fn signed(f: f64) -> Signed {
    if f.abs() < EPSILON {
        Signed::Zero
    } else if f > 0.0 {
        Signed::Positive
    } else {
        Signed::Negative
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::portfolio::seed::{aapl_id, instruments};

    pub const PORTFOLIO_ID: &str = "test-portfolio";

    #[test]
    fn test_position_long() {
        let inst_map = instruments();
        let inst = inst_map.get(&aapl_id()).unwrap().clone();

        // open position
        let mut pos = Position::new(inst.as_ref(), Execution::new(aapl_id(), PORTFOLIO_ID, 98.0, 100.0, Side::Buy));
        pos.apply(Execution::new(aapl_id(), PORTFOLIO_ID, 102.0, 50.0, Side::Buy));
        pos.apply(Execution::new(aapl_id(), PORTFOLIO_ID, 104.0, 25.0, Side::Buy));

        assert_eq!(pos.qty, 175.0);
        assert_eq!(pos.avg_px, 100.0);
        assert_eq!(pos.position_side, PositionSide::Long);
    }

    #[test]
    fn test_reduce_position() {
        let inst_map = instruments();
        let inst = inst_map.get(&aapl_id()).unwrap().clone();

        // Long aapl 175 @ 100
        let mut pos = Position::new(inst.as_ref(), Execution::new(aapl_id(), PORTFOLIO_ID, 100.0, 175.0, Side::Buy));

        // Reduce sell aapl 75 @ 110 -> 100 long remaining
        pos.apply(Execution::new(aapl_id(), PORTFOLIO_ID, 110.0, 75.0, Side::Sell));

        // PnL (110 - 100) * 75 = 750
        assert_eq!(pos.qty, 100.0);
        assert_eq!(pos.realized_pnl, 750.0);
    }
}
