use crate::conductor::types::{ConductorEvent, ModelTier};
use std::time::Instant;

/// Buffers incoming events and decides when to trigger an AI decision.
/// Prevents calling the API on every single hook event.
pub struct EventAggregator {
    buffer: Vec<ConductorEvent>,
    last_decision_at: Option<Instant>,
    min_interval: std::time::Duration,
    max_buffer: usize,
}

impl EventAggregator {
    pub fn new(min_interval_secs: u64, max_buffer: usize) -> Self {
        Self {
            buffer: Vec::new(),
            last_decision_at: None,
            min_interval: std::time::Duration::from_secs(min_interval_secs),
            max_buffer,
        }
    }

    /// Push an event. Returns Some((events, tier)) if a decision should be triggered.
    pub fn push(&mut self, event: ConductorEvent) -> Option<(Vec<ConductorEvent>, ModelTier)> {
        // Determine trigger urgency from event type
        let trigger = match &event {
            // High-signal: phase completed -> decide what's next (Haiku is enough)
            ConductorEvent::PhaseCompleted { .. } => Some(ModelTier::Haiku),
            // Error or unexpected end -> needs deeper analysis (Sonnet)
            ConductorEvent::SessionEnded { reason, .. } if reason != "completed" => {
                Some(ModelTier::Sonnet)
            }
            ConductorEvent::ErrorDetected { .. } => Some(ModelTier::Sonnet),
            // Everything else: buffer
            _ => None,
        };

        self.buffer.push(event);

        // Immediate trigger for high-signal events
        if let Some(tier) = trigger {
            return Some((self.drain(), tier));
        }

        // Buffer overflow trigger
        if self.buffer.len() >= self.max_buffer {
            return Some((self.drain(), ModelTier::Haiku));
        }

        None
    }

    /// Called by the periodic timer. Returns buffered events if interval has elapsed.
    pub fn tick(&mut self) -> Option<(Vec<ConductorEvent>, ModelTier)> {
        if self.buffer.is_empty() {
            return None;
        }
        if let Some(last) = self.last_decision_at {
            if last.elapsed() < self.min_interval {
                return None;
            }
        }
        Some((self.drain(), ModelTier::Haiku))
    }

    fn drain(&mut self) -> Vec<ConductorEvent> {
        self.last_decision_at = Some(Instant::now());
        self.buffer.drain(..).collect()
    }
}
