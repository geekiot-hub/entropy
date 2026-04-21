use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct PopupState {
    epochs: HashMap<&'static str, u64>,
    open: HashSet<&'static str>,
}

impl PopupState {
    pub fn on_open(&mut self, key: &'static str) {
        if self.open.insert(key) {
            *self.epochs.entry(key).or_insert(0) += 1;
        }
    }

    pub fn on_close(&mut self, key: &'static str) {
        self.open.remove(key);
    }

    pub fn begin_frame(&mut self, key: &'static str, is_open: bool) {
        if is_open {
            self.on_open(key);
        } else {
            self.on_close(key);
        }
    }

    pub fn id(&self, key: &'static str) -> egui::Id {
        egui::Id::new((key, self.epochs.get(key).copied().unwrap_or(0)))
    }
}
