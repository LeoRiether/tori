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
pub struct FilteredList<'a, T: 'a, St: Selectable> {
    pub items: Vec<&'a T>,
    pub state: St,
}

impl<'a, T: 'a, St: Selectable> FilteredList<'a, T, St> {
    pub fn filter<F>(&mut self, items: &'a [T], pred: F)
    where
        F: Fn(&&'a T) -> bool,
    {
        self.items = items.iter().filter(pred).collect();
        self.state
            .select(if self.items.is_empty() { None } else { Some(0) });
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

    pub fn selected_item(&self) -> Option<&'a T> {
        self.state.selected().map(|i| self.items[i])
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
