use crate::{
    position::Position,
    rectangle::{Border, BorderDirection, Rectangle},
    screen::Dimension,
    themes::Color,
};

use my_proc_macros::hex;
#[cfg(test)]
use ropey::Rope;

#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub rows: Vec<Vec<Cell>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cell {
    pub symbol: String,
    pub foreground_color: Color,
    pub background_color: Color,
    pub undercurl: Option<Color>,
}

impl Cell {
    #[cfg(test)]
    fn from_char(c: char) -> Self {
        use my_proc_macros::hex;

        Cell {
            symbol: c.to_string(),
            foreground_color: hex!("#ffffff"),
            background_color: hex!("#ffffff"),
            undercurl: None,
        }
    }

    fn apply_update(&self, update: CellUpdate) -> Cell {
        Cell {
            symbol: update.symbol.unwrap_or(self.symbol.clone()),
            foreground_color: update
                .style
                .foreground_color
                .unwrap_or(self.foreground_color),
            background_color: update
                .style
                .background_color
                .unwrap_or(self.background_color),
            undercurl: update.style.undercurl.or(self.undercurl),
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            symbol: " ".to_string(),
            foreground_color: hex!("#ffffff"),
            background_color: hex!("#ffffff"),
            undercurl: None,
        }
    }
}

pub struct CellUpdate {
    pub position: Position,
    pub symbol: Option<String>,
    pub style: Style,
}

impl CellUpdate {
    pub fn new(position: Position) -> Self {
        CellUpdate {
            position,
            symbol: None,
            style: Style::default(),
        }
    }

    pub fn symbol(self, symbol: String) -> Self {
        CellUpdate {
            symbol: Some(symbol),
            ..self
        }
    }

    pub fn background_color(self, background_color: Color) -> Self {
        CellUpdate {
            style: self.style.background_color(background_color),
            ..self
        }
    }

    pub fn foreground_color(self, foreground_color: Color) -> Self {
        CellUpdate {
            style: self.style.foreground_color(foreground_color),

            ..self
        }
    }

    pub fn subtract_vertical_offset(self, scroll_offset: usize) -> Option<CellUpdate> {
        if scroll_offset > self.position.line {
            None
        } else {
            Some(CellUpdate {
                position: Position {
                    line: self.position.line - scroll_offset,
                    ..self.position
                },
                ..self
            })
        }
    }

    pub fn undercurl(self, color: Option<Color>) -> CellUpdate {
        CellUpdate {
            style: self.style.undercurl(color),
            ..self
        }
    }

    pub fn style(self, style: Style) -> CellUpdate {
        CellUpdate { style, ..self }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct PositionedCell {
    pub cell: Cell,
    pub position: Position,
}

impl Grid {
    /// The `new_grid` need not be the same size as the old grid (`self`).
    pub fn diff(&self, new_grid: &Grid) -> Vec<PositionedCell> {
        let mut cells = vec![];
        for (row_index, new_row) in new_grid.rows.iter().enumerate() {
            for (column_index, new_cell) in new_row.iter().enumerate() {
                match self
                    .rows
                    .get(row_index)
                    .and_then(|old_row| old_row.get(column_index))
                {
                    Some(old_cell) if new_cell == old_cell => {
                        // Do nothing
                    }
                    // Otherwise
                    _ => cells.push(PositionedCell {
                        cell: new_cell.clone(),
                        position: Position {
                            line: row_index,
                            column: column_index,
                        },
                    }),
                }
            }
        }
        cells
    }

    pub fn new(dimension: Dimension) -> Grid {
        let mut cells: Vec<Vec<Cell>> = vec![];
        cells.resize_with(dimension.height.into(), || {
            let mut cells = vec![];
            cells.resize_with(dimension.width.into(), Cell::default);
            cells
        });
        Grid { rows: cells }
    }

    pub fn to_positioned_cells(&self) -> Vec<PositionedCell> {
        let mut cells = vec![];
        for (row_index, row) in self.rows.iter().enumerate() {
            for (column_index, cell) in row.iter().enumerate() {
                cells.push(PositionedCell {
                    cell: cell.clone(),
                    position: Position {
                        line: row_index,
                        column: column_index,
                    },
                })
            }
        }

        cells
    }

    #[cfg(test)]
    fn from_text(dimension: Dimension, text: &str) -> Grid {
        Grid::from_rope(dimension, &Rope::from_str(text))
    }

    #[cfg(test)]
    fn from_rope(dimension: Dimension, rope: &Rope) -> Grid {
        let mut grid = Grid::new(dimension);

        rope.lines().enumerate().for_each(|(row_index, line)| {
            line.chars()
                .enumerate()
                .for_each(|(column_index, character)| {
                    grid.rows[row_index][column_index] = Cell {
                        symbol: character.to_string(),
                        ..Cell::default()
                    }
                })
        });

        grid
    }

    pub fn update(self, other: &Grid, rectangle: &Rectangle) -> Grid {
        let mut grid = self;
        for (row_index, rows) in other.rows.iter().enumerate() {
            for (column_index, cell) in rows.iter().enumerate() {
                let row = row_index + rectangle.origin.line;
                let column = column_index + rectangle.origin.column;
                if row < grid.rows.len() && column < grid.rows[row].len() {
                    grid.rows[row][column] = cell.clone();
                }
            }
        }
        grid
    }

    pub fn merge_horizontal(self, right: Grid) -> Grid {
        let dimension = self.dimension();
        assert_eq!(dimension.height, right.dimension().height);

        let mut left = self;
        for (row_index, rows) in left.rows.iter_mut().enumerate() {
            rows.extend(right.rows[row_index].clone());
        }
        left
    }

    pub fn set_border(mut self, border: Border) -> Grid {
        let dimension = self.dimension();
        match border.direction {
            BorderDirection::Horizontal => {
                for i in 0..dimension.width.saturating_sub(border.start.column as u16) {
                    self.rows[border.start.line][border.start.column + i as usize] = Cell {
                        symbol: "─".to_string(),
                        foreground_color: hex!("#000000"),
                        ..Cell::default()
                    };
                }
            }
            BorderDirection::Vertical => {
                for i in 0..dimension.height.saturating_sub(border.start.line as u16) {
                    self.rows[border.start.line + i as usize][border.start.column] = Cell {
                        symbol: "│".to_string(),
                        foreground_color: hex!("#000000"),
                        ..Cell::default()
                    };
                }
            }
        }
        self
    }

    pub fn dimension(&self) -> Dimension {
        Dimension {
            height: self.rows.len() as u16,
            width: self.rows[0].len() as u16,
        }
    }

    pub fn set_line(self, row: usize, title: &str, style: Style) -> Grid {
        let mut grid = self;
        // Pad end with spaces
        let title = format!("{:<width$}", title, width = grid.dimension().width as usize);
        for (column_index, character) in title
            .chars()
            .take(grid.dimension().width as usize)
            .enumerate()
        {
            let default = Cell::default();
            grid.rows[row][column_index] = Cell {
                symbol: character.to_string(),
                foreground_color: style.foreground_color.unwrap_or(default.foreground_color),
                background_color: style.background_color.unwrap_or(default.background_color),
                ..default
            }
        }
        grid
    }

    pub fn apply_cell_update(mut self, update: CellUpdate) -> Grid {
        let Position { line, column } = update.position;
        if line < self.rows.len() && column < self.rows[line].len() {
            self.rows[line][column] = self.rows[line][column].apply_update(update);
        }
        self
    }

    pub fn apply_cell_updates(self, updates: Vec<CellUpdate>) -> Grid {
        updates
            .into_iter()
            .fold(self, |grid, update| grid.apply_cell_update(update))
    }

    pub fn content(&self) -> String {
        self.rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| cell.symbol.clone())
                    .collect::<Vec<String>>()
                    .join("")
            })
            .map(|line| line.replace('\n', " "))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[derive(Default, Clone, Copy)]
pub struct Style {
    pub foreground_color: Option<Color>,
    pub background_color: Option<Color>,
    pub undercurl: Option<Color>,
}

impl Style {
    pub const fn new() -> Style {
        Style {
            foreground_color: None,
            background_color: None,
            undercurl: None,
        }
    }

    pub const fn foreground_color(self, color: Color) -> Style {
        Style {
            foreground_color: Some(color),
            ..self
        }
    }

    pub const fn background_color(self, color: Color) -> Style {
        Style {
            background_color: Some(color),
            ..self
        }
    }

    pub const fn undercurl(self, color: Option<Color>) -> Style {
        Style {
            undercurl: color,
            ..self
        }
    }
}

#[cfg(test)]
mod test_grid {

    use pretty_assertions::assert_eq;

    use crate::{
        grid::{Cell, Grid, PositionedCell},
        position::Position,
        screen::Dimension,
    };

    #[test]
    fn diff_same_size() {
        let dimension = Dimension {
            height: 2,
            width: 4,
        };
        let old = Grid::from_text(dimension, "a\nbc");
        let new = Grid::from_text(dimension, "bc");
        let actual = old.diff(&new);
        let expected = vec![
            PositionedCell {
                position: Position { line: 0, column: 0 },
                cell: Cell::from_char('b'),
            },
            PositionedCell {
                position: Position { line: 0, column: 1 },
                cell: Cell::from_char('c'),
            },
            PositionedCell {
                position: Position { line: 1, column: 0 },
                cell: Cell::from_char(' '),
            },
            PositionedCell {
                position: Position { line: 1, column: 1 },
                cell: Cell::from_char(' '),
            },
        ];
        assert_eq!(actual, expected);
    }
}
