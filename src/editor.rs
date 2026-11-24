use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::io::stdout;
use std::path::Path;
use std::time::{Duration, Instant};

const BACKGROUND: Color = Color::Rgb(0, 0, 0);
const SURFACE: Color = Color::Rgb(119, 41, 42);
const ACCENT: Color = Color::Rgb(197, 64, 65);
const MUTED: Color = Color::Rgb(171, 54, 54);
const HEADING: Color = Color::Rgb(146, 48, 48);

pub fn run(target: &Path) -> Result<(), String> {
    enable_raw_mode().map_err(|error| error.to_string())?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|error| error.to_string())?;
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal, target);
    ratatui::restore();
    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = disable_raw_mode();
    result
}

fn run_app(terminal: &mut DefaultTerminal, target: &Path) -> Result<(), String> {
    let existing = crate::files::read_target(target)?;
    let mut lines: Vec<String> = existing.split('\n').map(str::to_string).collect();
    if lines.is_empty() {
        lines.push(String::new());
    }
    let mut cursor_row = lines.len() - 1;
    let mut cursor_col = lines[cursor_row].len();
    let mut status = "Existing content loaded".to_string();
    let mut typing_until: Option<Instant> = None;
    loop {
        terminal
            .draw(|frame| {
                draw(
                    frame,
                    target,
                    &lines,
                    cursor_row,
                    cursor_col,
                    &status,
                    typing_until.is_some_and(|until| Instant::now() < until),
                )
            })
            .map_err(|error| error.to_string())?;

        if !event::poll(Duration::from_millis(120)).map_err(|error| error.to_string())? {
            if typing_until.is_some_and(|until| Instant::now() >= until) {
                typing_until = None;
            }
            continue;
        }

        if let Event::Key(key) = event::read().map_err(|error| error.to_string())? {
            if key.code == KeyCode::Esc {
                break;
            }

            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
                status = match crate::files::write_blog(target, &join_lines(&lines)) {
                    Ok(_) => "Saved live".to_string(),
                    Err(error) => error,
                };
                continue;
            }

            let changed = match key.code {
                KeyCode::Char(character) => {
                    lines[cursor_row].insert(cursor_col, character);
                    cursor_col += character.len_utf8();
                    true
                }
                KeyCode::Enter => {
                    let remainder = lines[cursor_row].split_off(cursor_col);
                    cursor_row += 1;
                    cursor_col = 0;
                    lines.insert(cursor_row, remainder);
                    true
                }
                KeyCode::Backspace => {
                    if cursor_col > 0 {
                        cursor_col -= 1;
                        while !lines[cursor_row].is_char_boundary(cursor_col) {
                            cursor_col -= 1;
                        }
                        lines[cursor_row].remove(cursor_col);
                        true
                    } else if cursor_row > 0 {
                        cursor_row -= 1;
                        cursor_col = lines[cursor_row].len();
                        let remainder = lines.remove(cursor_row + 1);
                        lines[cursor_row].push_str(&remainder);
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Delete => {
                    if cursor_col < lines[cursor_row].len() {
                        lines[cursor_row].remove(cursor_col);
                        true
                    } else if cursor_row + 1 < lines.len() {
                        let remainder = lines.remove(cursor_row + 1);
                        lines[cursor_row].push_str(&remainder);
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Left => {
                    cursor_col = cursor_col.saturating_sub(1);
                    false
                }
                KeyCode::Right => {
                    cursor_col = (cursor_col + 1).min(lines[cursor_row].len());
                    false
                }
                KeyCode::Up => {
                    cursor_row = cursor_row.saturating_sub(1);
                    cursor_col = cursor_col.min(lines[cursor_row].len());
                    false
                }
                KeyCode::Down => {
                    if cursor_row + 1 < lines.len() {
                        cursor_row += 1;
                        cursor_col = cursor_col.min(lines[cursor_row].len());
                    }
                    false
                }
                KeyCode::Home => {
                    cursor_col = 0;
                    false
                }
                KeyCode::End => {
                    cursor_col = lines[cursor_row].len();
                    false
                }
                _ => false,
            };

            if changed {
                typing_until = Some(Instant::now() + Duration::from_millis(30));
                status = match crate::files::write_blog(target, &join_lines(&lines)) {
                    Ok(_) => "Saved live".to_string(),
                    Err(error) => error,
                };
            }
        }
    }

    Ok(())
}

fn join_lines(lines: &[String]) -> String {
    lines.join("\n")
}

fn draw(
    frame: &mut Frame,
    target: &Path,
    lines: &[String],
    cursor_row: usize,
    cursor_col: usize,
    status: &str,
    typing: bool,
) {
    frame.render_widget(
        Block::default().style(Style::default().bg(BACKGROUND)),
        frame.area(),
    );
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(frame.area());

    let filename = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("untitled");
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(SURFACE))
            .style(Style::default().bg(BACKGROUND)),
        chunks[0],
    );

    let inner_width = chunks[0].width.saturating_sub(2) as usize;
    let brand = "PANBAHAR";
    let left_width = filename.len();
    let brand_width = brand.len();
    let brand_start = inner_width.saturating_sub(brand_width) / 2;
    let left_gap = brand_start.saturating_sub(left_width);

    let header_spans = vec![
        Span::styled(
            filename.to_string(),
            Style::default().fg(Color::White).bg(BACKGROUND),
        ),
        Span::raw(" ".repeat(left_gap)),
        Span::styled(
            brand,
            Style::default()
                .fg(HEADING)
                .bg(BACKGROUND)
                .add_modifier(Modifier::BOLD),
        ),
    ];
    let header_line = Line::from(header_spans);
    let header_row = ratatui::layout::Rect::new(
        chunks[0].x + 1,
        chunks[0].y + 1,
        chunks[0].width.saturating_sub(2),
        1,
    );
    frame.render_widget(
        Paragraph::new(header_line).style(Style::default().bg(BACKGROUND)),
        header_row,
    );

    if typing {
        let emoji_area = ratatui::layout::Rect::new(
            chunks[0].x + chunks[0].width.saturating_sub(6),
            chunks[0].y + 1,
            5.min(chunks[0].width),
            1,
        );
        frame.render_widget(
            Paragraph::new("😎 ")
                .alignment(ratatui::layout::Alignment::Right)
                .style(Style::default().fg(ACCENT).bg(BACKGROUND)),
            emoji_area,
        );
    }

    let visible: Vec<Line> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let line_number = format!("{:>3} ", index + 1);
            Line::from(vec![
                Span::styled(line_number, Style::default().fg(SURFACE)),
                Span::styled(line.as_str(), Style::default().fg(ACCENT)),
            ])
        })
        .collect();
    frame.render_widget(
        Paragraph::new(visible)
            .style(Style::default().fg(ACCENT).bg(BACKGROUND))
            .block(
                Block::default()
                    .title(Span::styled(
                        " Write your blog ",
                        Style::default().fg(HEADING),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(SURFACE))
                    .style(Style::default().bg(BACKGROUND)),
            ),
        chunks[1],
    );

    let status_style = if status.starts_with("could") {
        Style::default()
            .fg(HEADING)
            .bg(BACKGROUND)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(MUTED).bg(BACKGROUND)
    };
    let status_line = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(status, status_style),
        Span::styled(
            "  •  Exit: Esc",
            Style::default().fg(Color::White).bg(BACKGROUND),
        ),
    ]);
    frame.render_widget(Paragraph::new(status_line), chunks[2]);

    let motto = "~Pehchan Kamyabi Ki !";
    let motto_width = motto.chars().count();
    let motto_area = ratatui::layout::Rect::new(
        chunks[2].x + chunks[2].width.saturating_sub(motto_width as u16 + 2),
        chunks[2].y,
        motto_width as u16 + 1,
        1,
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            motto,
            Style::default()
                .fg(HEADING)
                .bg(BACKGROUND)
                .add_modifier(Modifier::BOLD),
        ))),
        motto_area,
    );

    let editor_area = chunks[1];
    let line = &lines[cursor_row];
    let mut byte_col = cursor_col.min(line.len());
    while !line.is_char_boundary(byte_col) {
        byte_col -= 1;
    }
    let cursor_column = line[..byte_col].chars().count();
    let line_number_width = 4;
    let x = editor_area.x + 1 + line_number_width + cursor_column as u16;
    let y = editor_area.y + 1 + cursor_row as u16;
    if y < editor_area.y + editor_area.height - 1 {
        frame.set_cursor_position((x, y));
    }
}
