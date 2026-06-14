use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};
use std::io;
use std::process::Command;

pub fn main() -> io::Result<()> {
    let creds = load_credentials();
    let config = gcloud_config();
    let auth_list = gcloud_auth_list();

    let mut terminal = init_terminal()?;

    loop {
        terminal.draw(|frame| {
            let areas = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5), // credentials
                    Constraint::Length(5), // config
                    Constraint::Min(3),    // auth list
                ])
                .split(frame.area());

            // Credentials section
            let creds_rows: Vec<Row> = creds
                .iter()
                .map(|(k, v)| Row::new(vec![Cell::from(k.as_str()), Cell::from(v.as_str())]))
                .collect();
            let creds_table = Table::new(
                creds_rows,
                [Constraint::Percentage(30), Constraint::Percentage(70)],
            )
            .header(Row::new(vec![
                Cell::from("Key").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("Value").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Credentials (ADC)"),
            );
            frame.render_widget(creds_table, areas[0]);

            // Config section
            let config_rows: Vec<Row> = config
                .iter()
                .map(|(k, v)| Row::new(vec![Cell::from(k.as_str()), Cell::from(v.as_str())]))
                .collect();
            let config_table = Table::new(
                config_rows,
                [Constraint::Percentage(30), Constraint::Percentage(70)],
            )
            .header(Row::new(vec![
                Cell::from("Key").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("Value").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("gcloud config"),
            );
            frame.render_widget(config_table, areas[1]);

            // Auth list
            let auth_rows: Vec<Row> = auth_list
                .iter()
                .map(|line| Row::new(vec![Cell::from(line.as_str())]))
                .collect();
            let auth_table = Table::new(auth_rows, [Constraint::Percentage(100)])
                .header(Row::new(vec![
                    Cell::from("Accounts").style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("gcloud auth list"),
                );
            frame.render_widget(auth_table, areas[2]);
        })?;

        if quit_on_key()? {
            break;
        }
    }

    restore_terminal(&mut terminal)
}

fn load_credentials() -> Vec<(String, String)> {
    let path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!(
            "{}/.config/gcloud/application_default_credentials.json",
            home
        )
    });

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![("error".into(), format!("Cannot read {}", path))],
    };

    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => return vec![("error".into(), format!("Invalid JSON: {}", e))],
    };

    let mut rows = Vec::new();
    for key in ["type", "client_id", "quota_project_id", "universe_domain"] {
        if let Some(val) = v.get(key).and_then(|v| v.as_str()) {
            rows.push((key.to_string(), val.to_string()));
        }
    }
    rows
}

fn gcloud_config() -> Vec<(String, String)> {
    let output = Command::new("gcloud")
        .args(["config", "list", "--format=json"])
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return vec![("error".into(), "gcloud not available".into())],
    };

    let v: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(v) => v,
        Err(e) => return vec![("error".into(), format!("Parse error: {}", e))],
    };

    let mut rows = Vec::new();
    if let Some(core) = v.get("core") {
        for key in ["account", "project", "disable_usage_reporting"] {
            if let Some(val) = core.get(key).and_then(|v| v.as_str()) {
                rows.push((format!("core.{}", key), val.to_string()));
            }
        }
    }
    if let Some(compute) = v.get("compute") {
        if let Some(region) = compute.get("region").and_then(|v| v.as_str()) {
            rows.push(("compute.region".into(), region.to_string()));
        }
        if let Some(zone) = compute.get("zone").and_then(|v| v.as_str()) {
            rows.push(("compute.zone".into(), zone.to_string()));
        }
    }
    rows
}

fn gcloud_auth_list() -> Vec<String> {
    let output = Command::new("gcloud")
        .args(["auth", "list", "--format=value(account,status)"])
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect(),
        _ => vec!["gcloud not available".into()],
    }
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
}

fn quit_on_key() -> io::Result<bool> {
    if let Event::Key(key) = event::read()? {
        if key.code == KeyCode::Char('q') {
            return Ok(true);
        }
    }
    Ok(false)
}
