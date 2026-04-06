use crate::node::Node;
use crate::step::{Step, StepError, StepMap};
use crate::transform::Transform;

/// A group of steps that form a single undo entry.
#[derive(Debug, Clone)]
struct HistoryEntry {
    /// Inverted steps (in reverse order) to undo this entry.
    inverted_steps: Vec<Step>,
    /// Step maps for remapping through subsequent changes.
    maps: Vec<StepMap>,
    /// Timestamp of the last step in this entry (ms since epoch).
    timestamp: u64,
}

/// Undo/redo history with composition grouping.
#[derive(Debug, Clone)]
pub struct History {
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
    /// Composition window in milliseconds.
    compose_window_ms: u64,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            compose_window_ms: 500,
        }
    }

    pub fn with_compose_window(mut self, ms: u64) -> Self {
        self.compose_window_ms = ms;
        self
    }

    /// Record a transform's steps in the undo history.
    /// `timestamp` is milliseconds since epoch for composition grouping.
    pub fn record(&mut self, tr: &Transform, timestamp: u64) {
        if tr.is_empty() {
            return;
        }

        // Clear redo stack on new changes
        self.redo_stack.clear();

        // Collect inverted steps by re-applying to get inversions
        let mut inverted_steps = Vec::new();
        let mut maps = Vec::new();
        let mut doc = tr.start_doc().clone();
        for step in tr.steps() {
            if let Ok((new_doc, inv)) = step.apply(&doc) {
                inverted_steps.push(inv);
                maps.push(step.step_map());
                doc = new_doc;
            }
        }
        // Reverse so they can be applied in order to undo
        inverted_steps.reverse();

        // Check if we should compose with the previous entry
        let should_compose = self
            .undo_stack
            .last()
            .is_some_and(|prev| timestamp.saturating_sub(prev.timestamp) < self.compose_window_ms);

        if should_compose {
            let prev = self.undo_stack.last_mut().unwrap();
            // Remap previous inverted steps through new maps
            // Then prepend new inverted steps
            let mut combined = inverted_steps;
            combined.append(&mut prev.inverted_steps);
            prev.inverted_steps = combined;
            prev.maps.extend(maps);
            prev.timestamp = timestamp;
        } else {
            self.undo_stack.push(HistoryEntry {
                inverted_steps,
                maps,
                timestamp,
            });
        }
    }

    /// Create an undo transform. Returns None if nothing to undo.
    pub fn undo(&mut self, current_doc: &Node) -> Option<Result<Transform, StepError>> {
        let entry = self.undo_stack.pop()?;
        let mut tr = Transform::new(current_doc.clone());

        for step in &entry.inverted_steps {
            if let Err(e) = tr.add_step(step.clone()) {
                // Push entry back and return error
                self.undo_stack.push(entry);
                return Some(Err(e));
            }
        }

        // Record in redo stack
        let mut redo_inverted = Vec::new();
        let mut redo_maps = Vec::new();
        let mut doc = tr.start_doc().clone();
        for step in tr.steps() {
            if let Ok((new_doc, inv)) = step.apply(&doc) {
                redo_inverted.push(inv);
                redo_maps.push(step.step_map());
                doc = new_doc;
            }
        }
        redo_inverted.reverse();

        self.redo_stack.push(HistoryEntry {
            inverted_steps: redo_inverted,
            maps: redo_maps,
            timestamp: entry.timestamp,
        });

        Some(Ok(tr))
    }

    /// Create a redo transform. Returns None if nothing to redo.
    pub fn redo(&mut self, current_doc: &Node) -> Option<Result<Transform, StepError>> {
        let entry = self.redo_stack.pop()?;
        let mut tr = Transform::new(current_doc.clone());

        for step in &entry.inverted_steps {
            if let Err(e) = tr.add_step(step.clone()) {
                self.redo_stack.push(entry);
                return Some(Err(e));
            }
        }

        // Record in undo stack
        let mut undo_inverted = Vec::new();
        let mut undo_maps = Vec::new();
        let mut doc = tr.start_doc().clone();
        for step in tr.steps() {
            if let Ok((new_doc, inv)) = step.apply(&doc) {
                undo_inverted.push(inv);
                undo_maps.push(step.step_map());
                doc = new_doc;
            }
        }
        undo_inverted.reverse();

        self.undo_stack.push(HistoryEntry {
            inverted_steps: undo_inverted,
            maps: undo_maps,
            timestamp: entry.timestamp,
        });

        Some(Ok(tr))
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_depth(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_depth(&self) -> usize {
        self.redo_stack.len()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
