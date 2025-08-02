use std::{
    ops::Mul,
    time::{Duration, SystemTime},
};

use chess::Color;

#[cfg(feature = "logging")]
use crate::engine::search::MoveTimeLimitKind;

use crate::{engine::Engine, evaluator, uci::GoCommandConfig};

impl Engine {
    /// Move time limit.
    pub(super) fn stop_search_time(&mut self) -> bool {
        if let Some(soft_time) = self.search.soft_move_time {
            // Safe to unwrap since always both are set.
            let hard_time = self.search.hard_move_time.unwrap();

            let duration = SystemTime::now()
                .duration_since(self.search.start_time)
                .unwrap();

            // Always stop when hard limit is reached.
            if duration > hard_time {
                #[cfg(feature = "logging")]
                {
                    // Save to unwrap since log entry was added before.
                    self.search_log.time_limit_kind = Some(MoveTimeLimitKind::Hard);
                }

                return true;
            }

            // If the search was not volatile, abide to soft time limit.
            if !self.search.volatility && duration > soft_time {
                #[cfg(feature = "logging")]
                {
                    // Save to unwrap since log entry was added before.
                    self.search_log.time_limit_kind = Some(MoveTimeLimitKind::Soft);
                }

                return true;
            }
        }

        false
    }

    /// Checks if another iteration of PVS should be started based on the remaining time.
    pub(super) fn start_next_iteration(&mut self) -> bool {
        if self.search.soft_move_time.is_none() {
            return false;
        }

        // The CPW engine chronos modules measured that the next iteration roughly takes the same
        // time as all previous iterations. This assumption is also used here.
        // FIXME: check this assumption for this engine.
        //
        // time_used = now - start_time
        // predicted_time = time_used
        // time_left = move_time - predicted_time
        // if (predicted_time > time_left) return false
        //
        // Simplifies to:
        //     predicted_time > time_left
        // <=> time_used > (move_time - predicted_time)
        // <=> time_used > (move_time - time_used)
        // <=> 2 * time_used > move_time

        let move_time = if self.search.volatility {
            // Safe to unwrap since it was checked for non earlier.
            self.search.hard_move_time.unwrap()
        } else {
            // Safe to unwrap since both times are always set together.
            self.search.soft_move_time.unwrap()
        };

        SystemTime::now()
            .duration_since(self.search.start_time)
            .unwrap()
            .mul(2)
            < move_time
    }

    /// Sets the move time of the next search according to the received go command config.
    pub(super) fn set_move_time(&mut self, config: &GoCommandConfig) {
        // Decreases search time towards the beginning and end of the game and increases towards to
        // middle of the game. Game phase is calculated early game = 24 -> end game = 0.
        const MOVE_TIME_FACTORS: [(u64, u64); 25] = [
            (4, 5),  // 0.8
            (4, 5),  // 0.8
            (4, 5),  // 0.8
            (4, 5),  // 0.8
            (9, 10), // 0.9
            (9, 10), // 0.9
            (9, 10), // 0.9
            (9, 10), // 0.9
            (1, 1),  // 1
            (1, 1),  // 1
            (1, 1),  // 1
            (1, 1),  // 1
            (1, 1),  // 1
            (1, 1),  // 1
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (1, 1),  // 1
            (4, 5),  // 0.8
            (4, 5),  // 0.8
        ];

        if let Some((time, inc)) = match self.board.side_to_move() {
            Color::White => config
                .wtime
                .map(|time| (time, config.winc.unwrap_or(Duration::ZERO))),
            Color::Black => config
                .btime
                .map(|time| (time, config.binc.unwrap_or(Duration::ZERO))),
        } {
            // Just divide remaining time by 20 plus 3/4th of the increment.
            self.search.soft_move_time = Some((time / 20) + (3 * inc / 4));
        }

        // Set soft move time.
        self.search.hard_move_time = self.search.soft_move_time;

        // Use a safety margin to not lose by time.
        let move_time_safety_margin = 10;

        if let Some(time) = self.search.hard_move_time {
            let (num, denom) = MOVE_TIME_FACTORS[evaluator::game_phase(&self.board) as usize];

            // This should never truncate unless the time is about 1.8e19ms, which would be a bit
            // more than 570 million years.
            // Scale time by game phase.
            #[allow(clippy::cast_possible_truncation)]
            let time = (num * time.as_millis() as u64) / denom;

            // Subtract a safety margin from the soft time limit to avoid losing by time.
            let soft_move_time = if time > move_time_safety_margin {
                time - move_time_safety_margin
            } else {
                time
            };

            self.search.soft_move_time = Some(Duration::from_millis(soft_move_time));
            // Set hard time limit to be 150% of the soft time limit.
            self.search.hard_move_time = Some(Duration::from_millis(15 * soft_move_time / 10));
        }

        // Go movetime was set, always use the maximum time.
        if let Some(time) = config.move_time {
            // This should never truncate unless the time is about 1.8e19ms, which would be a bit
            // more than 570 million years.
            #[allow(clippy::cast_possible_truncation)]
            let millis = time.as_millis() as u64;

            let time = if millis > move_time_safety_margin {
                Duration::from_millis(millis - move_time_safety_margin)
            } else {
                time
            };

            self.search.soft_move_time = Some(time);
            self.search.hard_move_time = Some(time);
        }
    }
}
