use crate::node::Node;
use crate::step::{Step, StepError, StepMap};

/// Accumulates a sequence of steps and their position mappings.
#[derive(Debug, Clone)]
pub struct Transform {
    /// The original document before any steps.
    start_doc: Node,
    /// Document after each step (docs[i] = result of applying steps[i]).
    docs: Vec<Node>,
    /// The steps applied so far.
    steps: Vec<Step>,
    /// Position mappings for each step.
    maps: Vec<StepMap>,
}

impl Transform {
    pub fn new(doc: Node) -> Self {
        Self {
            start_doc: doc,
            docs: Vec::new(),
            steps: Vec::new(),
            maps: Vec::new(),
        }
    }

    /// The document before any steps were applied.
    pub fn start_doc(&self) -> &Node {
        &self.start_doc
    }

    /// The current document (after all applied steps).
    pub fn doc(&self) -> &Node {
        self.docs.last().unwrap_or(&self.start_doc)
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    pub fn maps(&self) -> &[StepMap] {
        &self.maps
    }

    /// Add and apply a step. Returns the inverted step on success.
    pub fn add_step(&mut self, step: Step) -> Result<Step, StepError> {
        let current = self.doc().clone();
        let (new_doc, inverted) = step.apply(&current)?;
        self.maps.push(step.step_map());
        self.steps.push(step);
        self.docs.push(new_doc);
        Ok(inverted)
    }

    /// Map a position through all steps in this transform.
    /// Bias: -1 maps left of insertions, 1 maps right.
    pub fn map_pos(&self, mut pos: usize, bias: i32) -> usize {
        for map in &self.maps {
            pos = map.map(pos, bias);
        }
        pos
    }

    /// Number of steps in this transform.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Whether any steps have been added.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

#[cfg(test)]
#[path = "transform_tests.rs"]
mod tests;
