use serde::{Deserialize, Serialize};

use super::{LifecycleError, LifecycleState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ControlKind {
    Pause,
    Resume,
    Cancel,
    Complete,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TargetControl {
    pub paused_from: Option<LifecycleState>,
    pub terminal: Option<LifecycleState>,
    pub last_seq: u64,
}

/// Default basic lifecycle rules. Callers own ordering, persistence and gates.
pub trait BasicLifecycle {
    fn control(&self) -> &TargetControl;
    fn control_mut(&mut self) -> &mut TargetControl;

    fn apply_control(
        &mut self,
        kind: ControlKind,
        prior_state: LifecycleState,
        evidence: Option<&str>,
        sequence: u64,
    ) -> Result<(), LifecycleError> {
        let control = self.control();
        let invalid = |message: String| LifecycleError::IllegalTransition(message);
        if let Some(terminal) = control.terminal {
            return Err(invalid(format!("event after terminal {terminal} control")));
        }
        let nonterminal = prior_state.is_resumable();
        match kind {
            ControlKind::Pause => {
                if control.paused_from.is_some() {
                    return Err(invalid("pause of an already paused target".into()));
                }
                if !nonterminal {
                    return Err(invalid(format!(
                        "pause requires a nonterminal prior state, got {prior_state}"
                    )));
                }
                self.control_mut().paused_from = Some(prior_state);
            }
            ControlKind::Resume => {
                let Some(paused_from) = control.paused_from else {
                    return Err(invalid("resume of a target that is not paused".into()));
                };
                if prior_state != paused_from {
                    return Err(invalid(format!(
                        "resume must restore {paused_from}, got {prior_state}"
                    )));
                }
                self.control_mut().paused_from = None;
            }
            ControlKind::Cancel => {
                let expected = control.paused_from.unwrap_or(prior_state);
                if !nonterminal || prior_state != expected {
                    return Err(invalid(format!(
                        "cancel requires nonterminal prior state {expected}, got {prior_state}"
                    )));
                }
                self.control_mut().paused_from = None;
                self.control_mut().terminal = Some(LifecycleState::Cancelled);
            }
            ControlKind::Complete => {
                if control.paused_from.is_some() {
                    return Err(invalid("completion of a paused target is rejected".into()));
                }
                if !nonterminal {
                    return Err(invalid(format!(
                        "complete requires a nonterminal prior state, got {prior_state}"
                    )));
                }
                if evidence.is_none_or(|value| value.trim().is_empty()) {
                    return Err(invalid("completion requires non-empty evidence".into()));
                }
                self.control_mut().terminal = Some(LifecycleState::Completed);
            }
        }
        self.control_mut().last_seq = sequence;
        Ok(())
    }
}

impl BasicLifecycle for TargetControl {
    fn control(&self) -> &TargetControl {
        self
    }

    fn control_mut(&mut self) -> &mut TargetControl {
        self
    }
}
