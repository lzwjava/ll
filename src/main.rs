mod gcloud;
mod uno;
mod word2vec;

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
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use std::io;
use sysinfo::{Disks, System};

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("");

    match cmd {
        "info" => run_info(),
        "gcloud" => gcloud::main(),
        "uno" => {
            let uno_args: Vec<String> = args[2..].to_vec();
            uno::main_with_args(&uno_args)
        }
        "word2vec" => {
            let w2v_args: Vec<String> = args[2..].to_vec();
            word2vec::main(&w2v_args)
        }
        _ => {
            eprintln!("Usage: ll <command>");
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  info     System info (CPU, memory, disks)");
            eprintln!("  gcloud   Google Cloud account & config");
            eprintln!("  uno      Arduino UNO serial monitor");
            eprintln!("  word2vec Word2Vec training");
            Ok(())
        }
    }
}

fn run_hello() -> io::Result<()> {
    let mut terminal = init_terminal()?;

    loop {
        terminal.draw(|frame| {
            let area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(frame.area());

            let hello = Paragraph::new("Hello")
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL).title("ll"));

            frame.render_widget(hello, area[1]);
        })?;

        if quit_on_key()? {
            break;
        }
    }

    restore_terminal(&mut terminal)
}

fn run_info() -> io::Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let disks = Disks::new_with_refreshed_list();

    let cpu_rows = cpu_rows(&sys);
    let mem_rows = mem_rows(&sys);
    let disk_rows = disk_rows(&disks);

    let mut terminal = init_terminal()?;

    loop {
        terminal.draw(|frame| {
            let areas = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(cpu_rows.len() as u16 + 3),
                    Constraint::Length(mem_rows.len() as u16 + 3),
                    Constraint::Min(disk_rows.len() as u16 + 3),
                ])
                .split(frame.area());

            frame.render_widget(make_table(&cpu_rows, "CPU", &["Key", "Value"]), areas[0]);
            frame.render_widget(make_table(&mem_rows, "Memory", &["Key", "Value"]), areas[1]);
            frame.render_widget(
                make_table(&disk_rows, "Disks", &["Mount", "Total", "Used", "Free"]),
                areas[2],
            );
        })?;

        if quit_on_key()? {
            break;
        }
    }

    restore_terminal(&mut terminal)
}

fn cpu_rows(sys: &System) -> Vec<Vec<String>> {
    let brand = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_default();
    let count = sys.cpus().len();
    let usage = sys.global_cpu_usage();
    vec![
        vec!["Brand".into(), brand],
        vec!["Cores".into(), count.to_string()],
        vec!["Usage".into(), format!("{:.1}%", usage)],
    ]
}

fn mem_rows(sys: &System) -> Vec<Vec<String>> {
    vec![
        vec!["Total".into(), fmt_bytes(sys.total_memory())],
        vec!["Used".into(), fmt_bytes(sys.used_memory())],
        vec!["Free".into(), fmt_bytes(sys.available_memory())],
    ]
}

fn disk_rows(disks: &Disks) -> Vec<Vec<String>> {
    disks
        .iter()
        .map(|d| {
            vec![
                d.mount_point().to_string_lossy().into_owned(),
                fmt_bytes(d.total_space()),
                fmt_bytes(d.total_space() - d.available_space()),
                fmt_bytes(d.available_space()),
            ]
        })
        .collect()
}

fn make_table<'a>(rows: &'a [Vec<String>], title: &'a str, headers: &'a [&'a str]) -> Table<'a> {
    let header = Row::new(headers.iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    }));

    let data_rows: Vec<Row> = rows
        .iter()
        .map(|r| Row::new(r.iter().map(|c| Cell::from(c.as_str()))))
        .collect();

    let widths: Vec<Constraint> = headers
        .iter()
        .map(|_| Constraint::Ratio(1, headers.len() as u32))
        .collect();

    Table::new(data_rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .column_spacing(2)
}

fn fmt_bytes(bytes: u64) -> String {
    const GB: u64 = 1 << 30;
    const MB: u64 = 1 << 20;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.0} MB", bytes as f64 / MB as f64)
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
