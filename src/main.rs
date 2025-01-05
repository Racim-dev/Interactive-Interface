use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};
use std::{io, time::Duration};

// Enum pour gérer le focus entre onglets et rectangles
#[derive(PartialEq, Clone, Copy)]
enum Focus {
    Tab(usize),    // Indique l'onglet sélectionné (index)
    Inner(usize),  // Indique le rectangle actif dans la grille
}

// État de l'application
struct AppState {
    focus: Focus,
    selected_item: usize, // Index de l'élément sélectionné dans la liste de R5
    list_state: ListState, // État de la liste pour R5
}

impl AppState {
    fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0)); // Définir la sélection initiale
        AppState {
            focus: Focus::Tab(0),
            selected_item: 0,
            list_state,
        }
    }
}

fn main() -> Result<(), io::Error> {
    // Activer le mode brut
    enable_raw_mode()?;

    // Initialisation du terminal
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // État initial
    let mut state = AppState::new();
    let total_tabs = 3;  // Nombre total d'onglets
    let total_rects = 9; // Nombre total de rectangles dans la grille (3x3)

    loop {
        terminal.draw(|frame| {
            // Layout principal : barre d'onglets + contenu principal
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),  // Hauteur pour la barre d'onglets
                        Constraint::Min(0),    // Reste pour le contenu principal
                    ]
                    .as_ref(),
                )
                .split(frame.area());

            // Barre d'onglets
            let tab_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    (0..total_tabs)
                        .map(|_| Constraint::Percentage(100 / total_tabs as u16))
                        .collect::<Vec<_>>(),
                )
                .split(main_chunks[0]);

            // Affichage des onglets
            for (i, tab) in tab_chunks.iter().enumerate() {
                let style = if let Focus::Tab(selected) = state.focus {
                    if selected == i {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }
                } else {
                    Style::default()
                };
                let tab_widget = Block::default()
                    .title(format!("Tab {}", i + 1))
                    .borders(Borders::ALL)
                    .style(style);
                frame.render_widget(tab_widget, *tab);
            }

            // Contenu principal : grille 3x3 de rectangles
            let inner_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(33); 3]) // 3 rangées
                .split(main_chunks[1]);

            let mut rect_index = 0;
            for row in inner_chunks.iter() {
                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(33); 3]) // 3 colonnes par rangée
                    .split(*row);

                for cell in row_chunks.iter() {
                    let cell_style = if let Focus::Inner(selected) = state.focus {
                        if selected == rect_index {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        }
                    } else {
                        Style::default()
                    };

                    if rect_index == 5 {
                        // R5 : Ajouter une liste spécifique
                        let items = vec![
                            ListItem::new("Item 1"),
                            ListItem::new("Item 2"),
                            ListItem::new("Item 3"),
                        ];
                        let list_widget = List::new(items)
                            .block(Block::default().borders(Borders::ALL).title("Liste sur R5"))
                            .highlight_style(Style::default().fg(Color::Cyan))
                            .highlight_symbol(">>");

                        // Afficher la liste avec l'élément sélectionné
                        frame.render_stateful_widget(list_widget, *cell, &mut state.list_state);
                    } else {
                        // Autres rectangles
                        let content = if rect_index == 4 {
                            "Texte ajouté sur R4 : Voici des informations spécifiques à ce rectangle."
                                .to_string()
                        } else {
                            format!("R{}", rect_index)
                        };

                        let cell_widget = Block::default()
                            .title(content)
                            .borders(Borders::ALL)
                            .style(cell_style);
                        frame.render_widget(cell_widget, *cell);
                    }

                    rect_index += 1;
                }
            }
        })?;

        // Gérer les événements clavier
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Right => {
                        match state.focus {
                            Focus::Tab(selected) => {
                                state.focus = Focus::Tab((selected + 1) % total_tabs);
                            }
                            Focus::Inner(selected) => {
                                state.focus = Focus::Inner((selected + 1) % total_rects);
                            }
                        }
                    }
                    KeyCode::Left => {
                        match state.focus {
                            Focus::Tab(selected) => {
                                state.focus = Focus::Tab((selected + total_tabs - 1) % total_tabs);
                            }
                            Focus::Inner(selected) => {
                                state.focus = Focus::Inner((selected + total_rects - 1) % total_rects);
                            }
                        }
                    }
                    KeyCode::Down => {
                        if let Focus::Inner(5) = state.focus {
                            // Descendre dans la liste si R5 est actif
                            let selected = state.list_state.selected().unwrap_or(0);
                            state.list_state.select(Some((selected + 1) % 3));
                        } else if let Focus::Inner(selected) = state.focus {
                            state.focus = Focus::Inner((selected + 3) % total_rects);
                        }
                    }
                    KeyCode::Up => {
                        if let Focus::Inner(5) = state.focus {
                            // Monter dans la liste si R5 est actif
                            let selected = state.list_state.selected().unwrap_or(0);
                            state.list_state
                                .select(Some((selected + 2) % 3)); // Navigation circulaire
                        } else if let Focus::Inner(selected) = state.focus {
                            state.focus = Focus::Inner((selected + total_rects - 3) % total_rects);
                        }
                    }
                    KeyCode::Enter => {
                        match state.focus {
                            Focus::Tab(_) => state.focus = Focus::Inner(0),
                            Focus::Inner(_) => state.focus = Focus::Tab(0),
                        }
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
    }

    // Restaurer le mode normal
    disable_raw_mode()?;
    Ok(())
}
