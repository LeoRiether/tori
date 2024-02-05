use tui::widgets::{ListState, TableState};

////////////////////////////////////
//        Selectable trait        //
////////////////////////////////////
pub trait Selectable {
    fn selected(&self) -> Option<usize>;
    fn select(&mut self, index: Option<usize>);
}

impl Selectable for ListState {
    fn selected(&self) -> Option<usize> {
        self.selected()
    }

    fn select(&mut self, index: Option<usize>) {
        self.select(index);
    }
}

impl Selectable for TableState {
    fn selected(&self) -> Option<usize> {
        self.selected()
    }

    fn select(&mut self, index: Option<usize>) {
        self.select(index);
    }
}

/////////////////////////////////
//        Filtered List        //
/////////////////////////////////
#[derive(Debug, Default)]
pub struct FilteredList<St: Selectable> {
    /// List of indices of the original list
    pub items: Vec<usize>,
    pub state: St,
}

impl<St: Selectable> FilteredList<St> {
    pub fn filter<T, P, S>(&mut self, items: &[T], pred: P, sorting: S)
    where
        P: Fn(&T) -> bool,
        S: Fn(usize, usize) -> std::cmp::Ordering,
    {
        let previous_selection = self.selected_item();

        self.items = (0..items.len()).filter(|&i| pred(&items[i])).collect();
        self.items.sort_by(|&i, &j| sorting(i, j));

        let new_selection = self
            .items
            .iter()
            // Search for the item that was previously selected
            .position(|&i| Some(i) == previous_selection)
            // If we don't find it, select the first item
            .or(if self.items.is_empty() { None } else { Some(0) });

        self.state.select(new_selection);
    }

    pub fn select_next(&mut self) {
        self.state.select(match self.state.selected() {
            Some(x) => Some(wrap_inc(x, self.items.len())),
            None if !self.items.is_empty() => Some(0),
            None => None,
        });
    }

    pub fn select_prev(&mut self) {
        self.state.select(match self.state.selected() {
            Some(x) => Some(wrap_dec(x, self.items.len())),
            None if !self.items.is_empty() => Some(0),
            None => None,
        });
    }

    pub fn select(&mut self, i: Option<usize>) {
        self.state.select(i);
    }

    pub fn selected_item(&self) -> Option<usize> {
        self.state.selected().map(|i| self.items[i])
    }

    pub fn iter(&self) -> impl Iterator<Item = &usize> {
        self.items.iter()
    }
}

fn wrap_inc(x: usize, modulo: usize) -> usize {
    if x == modulo - 1 {
        0
    } else {
        x + 1
    }
}

fn wrap_dec(x: usize, modulo: usize) -> usize {
    if x == 0 {
        modulo - 1
    } else {
        x - 1
    }
}
