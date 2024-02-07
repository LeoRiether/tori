use crate::{ui::Scrollbar, events::Action};

use super::{on, EventfulWidget, Listener, UIEvent};
use std::mem;
use tui::{
    prelude::*,
    widgets::{
        Block, BorderType, Borders, Paragraph, Row, StatefulWidget, Table, TableState, Widget, Wrap,
    },
};

#[derive(Default)]
pub struct List<'a, const C: usize> {
    title: String,
    highlight_style: Style,
    highlight_symbol: Option<&'a str>,
    border_style: Style,
    borders: Borders,
    state: TableState,
    items: Vec<[String; C]>,
    help_message: String,
    click_event: Option<Box<dyn Fn(usize) -> Action>>,
}

impl<'a, const C: usize> EventfulWidget<Action> for List<'a, C> {
    fn render(&mut self, area: Rect, buf: &mut Buffer, l: &mut Vec<Listener<Action>>) {
        let block = Block::default()
            .title(mem::take(&mut self.title))
            .borders(self.borders)
            .border_type(BorderType::Plain)
            .border_style(self.border_style);

        if self.items.is_empty() {
            let help = Paragraph::new(mem::take(&mut self.help_message))
                .wrap(Wrap { trim: true })
                .block(block)
                .style(Style::default().fg(Color::DarkGray));
            help.render(area, buf);
            return;
        }

        // Compute column widths
        let mut widths = vec![Constraint::Percentage(100)];
        for i in 1..C {
            let width = self.items.iter().map(|r| r[i].len()).max().unwrap_or(0);
            widths.push(Constraint::Min(width as u16 + 1));
        }

        // Render table
        let items_len = self.items.len();
        let items = mem::take(&mut self.items).into_iter().map(Row::new);
        let mut table = Table::new(items, widths)
            .block(block)
            .highlight_style(self.highlight_style);

        if let Some(symbol) = self.highlight_symbol {
            table = table.highlight_symbol(symbol);
        }

        StatefulWidget::render(table, area, buf, &mut self.state);

        // Render scrollbar
        if items_len > area.height as usize - 2 {
            let area = area.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            });
            Scrollbar::new(self.state.selected().unwrap_or(0) as u16, items_len as u16)
                .with_style(self.border_style)
                .render(area, buf);
        }

        // Event listeners
        if let Some(click_event) = &self.click_event {
            let offset = self.state.offset();
            let height = std::cmp::max(area.height as usize - 2, items_len);
            for i in offset..offset + height {
                let event = click_event(i);
                let rect = Rect {
                    x: area.x,
                    y: area.y + 1 + i as u16,
                    width: area.width - 2,
                    height: 1,
                };
                l.push(on(UIEvent::Click(rect), move |_| event.clone()));
            }
        }
    }
}

impl<'a, const C: usize> List<'a, C> {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    pub fn highlight_symbol(mut self, symbol: &'a str) -> Self {
        self.highlight_symbol = Some(symbol);
        self
    }

    pub fn border_style(mut self, border: Style) -> Self {
        self.border_style = border;
        self
    }

    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    pub fn state(mut self, state: TableState) -> Self {
        self.state = state;
        self
    }

    pub fn get_state(&self) -> TableState {
        self.state.clone()
    }

    pub fn items(mut self, items: Vec<[String; C]>) -> Self {
        self.items = items;
        self
    }

    pub fn help_message(mut self, help_message: String) -> Self {
        self.help_message = help_message;
        self
    }

    pub fn click_event(mut self, action: impl Fn(usize) -> Action + 'static) -> Self {
        self.click_event = Some(Box::new(action));
        self
    }
}
