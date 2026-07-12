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
    symbols::Marker,
    text::Span,
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph,
    },
};
use std::io;
use std::io::BufRead;
use std::time::Duration;

const PORT: &str = "/dev/ttyUSB0";
const BAUD: u32 = 9600;
const MAX_POINTS: usize = 200;

fn open_port() -> io::Result<Box<dyn serialport::SerialPort>> {
    serialport::new(PORT, BAUD)
        .timeout(Duration::from_millis(500))
        .open()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Cannot open {} at {} baud: {}", PORT, BAUD, e)))
}

pub fn main_with_args(args: &[String]) -> io::Result<()> {
    let sub = args.first().map(String::as_str).unwrap_or("");

    match sub {
        "status" => run_status(),
        "listen" => run_listen(),
        _ => {
            eprintln!("Usage: ll uno <subcommand>");
            eprintln!();
            eprintln!("Subcommands:");
            eprintln!("  status   TUI: live readings + sparkline");
            eprintln!("  listen   Raw dump to stdout");
            Ok(())
        }
    }
}

pub fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    main_with_args(&args[2..])
}

/// Raw dump: just print readings as they arrive.
fn run_listen() -> io::Result<()> {
    let mut port = open_port()?;
    let reader = io::BufReader::new(&mut port);
    println!("Listening on {} at {} baud. Ctrl-C to stop.", PORT, BAUD);
    for line in reader.lines() {
        match line {
            Ok(l) => println!("{}", l.trim()),
            Err(e) => eprintln!("read error: {}", e),
        }
    }
    Ok(())
}

/// TUI: live readings with sparkline/chart.
fn run_status() -> io::Result<()> {
    let port = open_port()?;
    enable_raw_mode().map_err(|e| {
        // ENXIO (code 6) typically means no TTY — happens when run through
        // `sg dialout` which detaches stdin. Recommend adding user to dialout group.
        if e.raw_os_error() == Some(6) {
            eprintln!(
                "error: need a TTY for the TUI.\n\
                 Try:  sudo usermod -a -G dialout $USER   (then log out & back in)\n\
                 Then: cargo run -- uno status\n\
                 Or:   cargo run -- uno listen  (plain text mode)"
            );
        }
        e
    })?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut readings: Vec<f64> = Vec::with_capacity(MAX_POINTS);
    let mut last_value: f64 = 0.0;

    // Wrap in BufReader for line reading
    let mut reader = io::BufReader::new(port);

    loop {
        // Read as many lines as available
        loop {
            let mut single = String::new();
            match reader.read_line(&mut single) {
                Ok(0) => break, // timeout, no data
                Ok(_) => {
                    let trimmed = single.trim();
                    if let Ok(val) = trimmed.parse::<f64>() {
                        last_value = val;
                        readings.push(val);
                        if readings.len() > MAX_POINTS {
                            readings.remove(0);
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => break,
                Err(e) => {
                    eprintln!("read error: {}", e);
                    break;
                }
            }
        }

        terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // header
                    Constraint::Length(3),  // current value
                    Constraint::Min(8),     // chart
                    Constraint::Length(5),  // stats
                ])
                .split(area);

            // --- Header ---
            let header = Paragraph::new(format!(
                " Arduino UNO — {} @ {} baud ",
                PORT, BAUD
            ))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
            frame.render_widget(header, chunks[0]);

            // --- Current value ---
            let val_style = if last_value > 30.0 {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let val_text = Paragraph::new(Span::styled(
                format!("{:.0}", last_value),
                val_style,
            ))
            .block(Block::default().borders(Borders::ALL).title("Amplitude"))
            .style(Style::default());
            frame.render_widget(val_text, chunks[1]);

            // --- Chart ---
            if !readings.is_empty() {
                let min_val = readings.iter().cloned().fold(f64::MAX, f64::min).max(0.0);
                let max_val = readings.iter().cloned().fold(f64::MIN, f64::max).max(1.0);
                let range = (max_val - min_val).max(1.0);

                let data_points: Vec<(f64, f64)> = readings
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| (i as f64, v))
                    .collect();

                let dataset = Dataset::default()
                    .marker(Marker::Dot)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Green))
                    .data(&data_points);

                let chart = Chart::new(vec![dataset])
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Signal"),
                    )
                    .x_axis(
                        Axis::default()
                            .bounds([0.0, MAX_POINTS as f64]),
                    )
                    .y_axis(
                        Axis::default()
                            .bounds([min_val, max_val + range * 0.1])
                            .labels(vec![
                                Span::raw(format!("{:.0}", min_val)),
                                Span::raw(format!("{:.0}", max_val)),
                            ]),
                    );
                frame.render_widget(chart, chunks[2]);
            } else {
                let waiting = Paragraph::new(" Waiting for data... ")
                    .style(Style::default().fg(Color::Gray))
                    .block(Block::default().borders(Borders::ALL).title("Signal"));
                frame.render_widget(waiting, chunks[2]);
            }

            // --- Stats ---
            let stats = if !readings.is_empty() {
                let min = readings.iter().cloned().fold(f64::MAX, f64::min);
                let max = readings.iter().cloned().fold(f64::MIN, f64::max);
                let avg = readings.iter().sum::<f64>() / readings.len() as f64;
                format!(
                    " Samples: {}  |  Min: {:.0}  |  Max: {:.0}  |  Avg: {:.1}  |  Last: {:.0}",
                    readings.len(),
                    min,
                    max,
                    avg,
                    last_value,
                )
            } else {
                " Waiting...".into()
            };
            let stats_para = Paragraph::new(stats)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Stats"));
            frame.render_widget(stats_para, chunks[3]);
        })?;

        // Check for quit
        if crossterm::event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
