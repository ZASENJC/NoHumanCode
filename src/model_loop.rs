use tokio_util::sync::CancellationToken;

/// Request admission for the embedded model loop. The standalone loop lives in
/// `codacli_core::agent_loop`.
pub(crate) struct ModelLoop {
    rounds: std::ops::Range<usize>,
}

pub(crate) enum Stop {
    Cancelled,
    Limit,
}

impl ModelLoop {
    pub(crate) fn new(max_requests: usize) -> Self {
        Self {
            rounds: 0..max_requests,
        }
    }

    pub(crate) fn next(&mut self, cancel: &CancellationToken) -> Result<usize, Stop> {
        if cancel.is_cancelled() {
            return Err(Stop::Cancelled);
        }
        self.rounds.next().ok_or(Stop::Limit)
    }

    pub(crate) fn can_continue(&self) -> bool {
        !self.rounds.is_empty()
    }
}
