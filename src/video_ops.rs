//! Common video operations — `ll video <command> [args] [options]`.
//!
//! Video decoding/transcoding in pure Rust is not realistic, so these commands
//! drive the system `ffmpeg` / `ffprobe` binaries (they must be on `PATH`).
//! Every command prints the exact ffmpeg command line it runs, and `--dry-run`
//! prints it without executing anything.
//!
//! Covered operations: probe info, screenshots, frame extraction, trimming,
//! joining, audio extract/mute/volume, resize/crop/rotate/flip/speed, gif
//! round-trips, contact sheets and generic transcoding.

use crate::cli::{arg_or, err, merge, normalize, parse_f64, parse_opts, parse_u32, pos, Opts};
use serde_json::Value;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn main_with_args(args: &[String]) -> io::Result<()> {
    let sub = args.first().map(String::as_str).unwrap_or("");
    let rest: &[String] = if args.is_empty() { &[] } else { &args[1..] };

    match sub {
        "" | "help" | "--help" | "-h" => {
            usage();
            Ok(())
        }
        "list" | "--list" => {
            list();
            Ok(())
        }
        _ => match find_cmd(sub) {
            Some(cmd) if args.get(1).map(String::as_str) == Some("--help") => {
                println!("ll video {}", cmd.usage);
                println!("  {}", cmd.desc);
                println!();
                println!("  Times accept 90, 1:30 or 01:02:03.5.  --dry-run prints the ffmpeg command only.");
                Ok(())
            }
            Some(cmd) => dispatch(cmd.name, rest),
            None => {
                eprintln!("Unknown video command: {sub}\n");
                list();
                Ok(())
            }
        },
    }
}

fn usage() {
    println!("ll video — common video operations (ffmpeg on the outside)");
    println!();
    println!("Usage: ll video <command> [args] [options]");
    println!();
    println!("  ll video list               All commands");
    println!("  ll video <command> --help   Options for one command");
    println!();
    println!("Examples:");
    println!("  ll video info clip.mp4 --json");
    println!("  ll video screenshot clip.mp4 shot.png -t 1:30");
    println!("  ll video frames clip.mp4 shots/frame --fps 1");
    println!("  ll video trim clip.mp4 part.mp4 -s 10 -e 25");
    println!("  ll video concat movie.mp4 intro.mp4 body.mp4 outro.mp4");
    println!("  ll video gif clip.mp4 loop.gif -s 12 -w 480");
    println!("  ll video sprite clip.mp4 sheet.png --cols 4 --rows 3");
    println!("  ll video convert clip.mov clip.mp4 --codec h264 --crf 20");
}

fn list() {
    let mut group = "";
    for c in CMDS {
        if c.group != group {
            group = c.group;
            if !group.is_empty() {
                println!();
            }
            println!("{group}");
        }
        println!("  {:<58} {}", c.usage, c.desc);
    }
    println!();
    println!("Every command also accepts --dry-run to print the ffmpeg command only.");
}

struct VCmd {
    name: &'static str,
    usage: &'static str,
    group: &'static str,
    desc: &'static str,
}

static CMDS: &[VCmd] = &[
    VCmd { name: "info", usage: "info <file>... [--json]", group: "Inspect", desc: "Probe container, streams, codecs, fps, bitrate" },
    VCmd { name: "screenshot", usage: "screenshot <in> <out> [-t TIME] [-q N]", group: "Cut & join", desc: "Save a single frame as png/jpg/webp" },
    VCmd { name: "frames", usage: "frames <in> <prefix> [--fps N | --every N] [-s T] [-d DUR]", group: "Cut & join", desc: "Extract an image sequence" },
    VCmd { name: "trim", usage: "trim <in> <out> [-s START] [-e END | -d DUR] [--reencode]", group: "Cut & join", desc: "Cut a segment (stream copy by default)" },
    VCmd { name: "concat", usage: "concat <out> <in...> [--reencode]", group: "Cut & join", desc: "Join clips end to end" },
    VCmd { name: "reverse", usage: "reverse <in> <out>", group: "Cut & join", desc: "Play backwards (loads whole clip in RAM)" },
    VCmd { name: "audio", usage: "audio <in> <out>", group: "Audio", desc: "Extract the audio track (mp3/m4a/wav/flac/opus)" },
    VCmd { name: "mute", usage: "mute <in> <out>", group: "Audio", desc: "Drop the audio track, copy video" },
    VCmd { name: "volume", usage: "volume <in> <out> -v FACTOR|dB", group: "Audio", desc: "Adjust loudness, e.g. -v 0.5 or -v -6dB" },
    VCmd { name: "resize", usage: "resize <in> <out> (-s WxH | -w W | -h H | --percent N)", group: "Transform", desc: "Scale, aspect preserved with -w/-h" },
    VCmd { name: "crop", usage: "crop <in> <out> -g WxH+X+Y", group: "Transform", desc: "Cut a rectangle out of the frame" },
    VCmd { name: "rotate", usage: "rotate <in> <out> -a 90|180|270", group: "Transform", desc: "Quarter turns" },
    VCmd { name: "flip", usage: "flip <in> <out> --horizontal | --vertical", group: "Transform", desc: "Mirror the picture" },
    VCmd { name: "speed", usage: "speed <in> <out> -v FACTOR", group: "Transform", desc: "2x faster / 0.5x slower, audio follows" },
    VCmd { name: "convert", usage: "convert <in> <out> [--codec C] [--crf N] [--preset P]", group: "Convert & export", desc: "Generic transcode, also --fps and --size" },
    VCmd { name: "gif", usage: "gif <in> <out.gif> [-s FPS] [-w WIDTH]", group: "Convert & export", desc: "Animated gif with an optimized palette" },
    VCmd { name: "gif2video", usage: "gif2video <in.gif> <out>", group: "Convert & export", desc: "Turn a gif into mp4/webm" },
    VCmd { name: "sprite", usage: "sprite <in> <out.png> [--cols N --rows N --interval S]", group: "Convert & export", desc: "Contact sheet / thumbnail grid" },
];

const ALIASES: &[(&str, &str)] = &[
    ("frame", "screenshot"),
    ("shot", "screenshot"),
    ("snap", "screenshot"),
    ("extractframes", "frames"),
    ("sequence", "frames"),
    ("join", "concat"),
    ("stripaudio", "mute"),
    ("extractaudio", "audio"),
    ("scale", "resize"),
    ("transcode", "convert"),
    ("contactsheet", "sprite"),
    ("sheet", "sprite"),
    ("tile", "sprite"),
    ("giffy", "gif"),
];

fn find_cmd(name: &str) -> Option<&'static VCmd> {
    let mut key = normalize(name);
    if let Some((_, target)) = ALIASES.iter().find(|(a, _)| normalize(a) == key) {
        key = target.to_string();
    }
    CMDS.iter().find(|c| c.name == key)
}

fn dispatch(name: &str, args: &[String]) -> io::Result<()> {
    match name {
        "info" => cmd_info(args),
        "screenshot" => cmd_screenshot(args),
        "frames" => cmd_frames(args),
        "trim" => cmd_trim(args),
        "concat" => cmd_concat(args),
        "reverse" => cmd_reverse(args),
        "audio" => cmd_audio(args),
        "mute" => cmd_mute(args),
        "volume" => cmd_volume(args),
        "resize" => cmd_resize(args),
        "crop" => cmd_crop(args),
        "rotate" => cmd_rotate(args),
        "flip" => cmd_flip(args),
        "speed" => cmd_speed(args),
        "convert" => cmd_convert(args),
        "gif" => cmd_gif(args),
        "gif2video" => cmd_gif2video(args),
        "sprite" => cmd_sprite(args),
        other => err(format!("internal: no handler for '{other}'")),
    }
}

// ─── ffmpeg plumbing ─────────────────────────────────────────────────────────

fn tool_missing(bin: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "{bin} not found in PATH — install ffmpeg (https://ffmpeg.org/download.html),\n  e.g. `sudo apt install ffmpeg`, `brew install ffmpeg` or `winget install ffmpeg`"
        ),
    )
}

fn require(bin: &str) -> io::Result<()> {
    let ok = Command::new(bin)
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        Ok(())
    } else {
        Err(tool_missing(bin))
    }
}

/// Run ffmpeg with the standard global options prepended.
fn run_ffmpeg(args: &[String], dry_run: bool) -> io::Result<()> {
    require("ffmpeg")?;
    println!("$ ffmpeg -hide_banner -loglevel error -y {}", args.join(" "));
    if dry_run {
        return Ok(());
    }
    let status = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-y"])
        .args(args)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        err(format!("ffmpeg failed ({status}) — see the message above"))
    }
}

/// Run ffprobe and return its stdout.
fn run_ffprobe(args: &[String]) -> io::Result<String> {
    require("ffprobe")?;
    let out = Command::new("ffprobe")
        .args(["-hide_banner", "-loglevel", "error"])
        .args(args)
        .output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return err(format!("ffprobe failed: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

// ─── info ────────────────────────────────────────────────────────────────────

const INFO_VALS: &[&str] = &[];

fn cmd_info(args: &[String]) -> io::Result<()> {
    let o = parse_opts(args, INFO_VALS, &["--json"])?;
    if o.pos.is_empty() {
        return err("info: give at least one file (ll video info clip.mp4)");
    }
    let json_mode = o.has(&["--json"]);
    for (i, file) in o.pos.iter().enumerate() {
        let probe = probe(file)?;
        if json_mode {
            println!(
                "{}",
                serde_json::to_string_pretty(&probe).map_err(io::Error::other)?
            );
        } else {
            if i > 0 {
                println!();
            }
            print!("{}", summarize(file, &probe));
        }
    }
    Ok(())
}

fn probe(file: &str) -> io::Result<Value> {
    let args = vec![
        "-print_format".into(),
        "json".into(),
        "-show_format".into(),
        "-show_streams".into(),
        file.to_string(),
    ];
    let out = run_ffprobe(&args)?;
    serde_json::from_str(&out).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("'{file}': {e}")))
}

fn summarize(file: &str, probe: &Value) -> String {
    let fmt = &probe["format"];
    let mut s = String::new();
    s.push_str(&format!("{file}\n"));

    let container = {
        let names: Vec<&str> = fmt["format_name"]
            .as_str()
            .unwrap_or("?")
            .split(',')
            .collect();
        let ext = ext_of(file);
        names.iter().find(|n| **n == ext).copied().unwrap_or(names[0])
    };
    let duration = fmt["duration"].as_str().and_then(|d| d.parse::<f64>().ok());
    let size = fmt["size"].as_str().and_then(|b| b.parse::<u64>().ok()).unwrap_or(0);
    let bitrate = fmt["bit_rate"].as_str().and_then(|b| b.parse::<u64>().ok()).unwrap_or(0);
    s.push_str(&format!(
        "  container  {container}{}{}{}\n",
        duration.map(|d| format!("  ·  {d:.2} s")).unwrap_or_default(),
        if size > 0 { format!("  ·  {}", fmt_bytes(size)) } else { String::new() },
        if bitrate > 0 { format!("  ·  {} kb/s", bitrate / 1000) } else { String::new() },
    ));

    if let Some(streams) = probe["streams"].as_array() {
        for st in streams {
            match st["codec_type"].as_str().unwrap_or("") {
                "video" => {
                    let w = st["width"].as_u64().unwrap_or(0);
                    let h = st["height"].as_u64().unwrap_or(0);
                    s.push_str(&format!(
                        "  video      {}  {w}x{h}{}{}{}\n",
                        st["codec_name"].as_str().unwrap_or("?"),
                        st["r_frame_rate"]
                            .as_str()
                            .and_then(fps_str)
                            .map(|f| format!("  {f} fps"))
                            .unwrap_or_default(),
                        st["pix_fmt"].as_str().map(|p| format!("  {p}")).unwrap_or_default(),
                        kbps(&st["bit_rate"]),
                    ));
                }
                "audio" => {
                    let layout = st["channel_layout"].as_str().unwrap_or("");
                    let channels = st["channels"].as_u64().unwrap_or(0);
                    s.push_str(&format!(
                        "  audio      {}  {} Hz  {}{}\n",
                        st["codec_name"].as_str().unwrap_or("?"),
                        st["sample_rate"].as_str().unwrap_or("?"),
                        if layout.is_empty() { format!("{channels} ch") } else { format!("{layout} ({channels} ch)") },
                        kbps(&st["bit_rate"]),
                    ));
                }
                "subtitle" => {
                    s.push_str(&format!("  subtitle   {}\n", st["codec_name"].as_str().unwrap_or("?")));
                }
                _ => {}
            }
        }
    }
    s
}

fn kbps(bit_rate: &Value) -> String {
    match bit_rate.as_str().and_then(|b| b.parse::<u64>().ok()) {
        Some(b) if b > 0 => format!("  {} kb/s", b / 1000),
        _ => String::new(),
    }
}

/// `"30000/1001"` -> `"29.97"`, `"25/1"` -> `"25"`.
fn fps_str(rate: &str) -> Option<String> {
    let (n, d) = rate.split_once('/')?;
    let (n, d): (f64, f64) = (n.parse().ok()?, d.parse().ok()?);
    if d == 0.0 {
        return None;
    }
    let fps = n / d;
    Some(if fps.fract().abs() < 0.005 {
        format!("{}", fps.round())
    } else {
        format!("{fps:.2}")
    })
}

fn fmt_bytes(b: u64) -> String {
    const KB: u64 = 1 << 10;
    const MB: u64 = 1 << 20;
    const GB: u64 = 1 << 30;
    if b >= GB {
        format!("{:.2} GB", b as f64 / GB as f64)
    } else if b >= MB {
        format!("{:.1} MB", b as f64 / MB as f64)
    } else {
        format!("{:.1} KB", b as f64 / KB as f64)
    }
}

/// Duration in seconds, if the file has one.
fn probe_duration(file: &str) -> io::Result<Option<f64>> {
    let out = run_ffprobe(&[
        "-show_entries".into(),
        "format=duration".into(),
        "-of".into(),
        "default=noprint_wrappers=1:nokey=1".into(),
        file.to_string(),
    ])?;
    Ok(out.trim().parse::<f64>().ok())
}

fn probe_has_audio(file: &str) -> io::Result<bool> {
    let out = run_ffprobe(&[
        "-select_streams".into(),
        "a:0".into(),
        "-show_entries".into(),
        "stream=codec_type".into(),
        "-of".into(),
        "csv=p=0".into(),
        file.to_string(),
    ])?;
    Ok(out.contains("audio"))
}

// ─── Value helpers ───────────────────────────────────────────────────────────

/// `90`, `1:30` and `01:02:03.5` all become seconds.
fn parse_time(s: &str) -> io::Result<f64> {
    let parts: Vec<&str> = s.trim().split(':').collect();
    let secs = match parts.len() {
        1 => parse_f64(s, "time")?,
        2 | 3 => {
            let mut total = 0.0;
            for p in parts {
                total = total * 60.0 + parse_f64(p, "time")?;
            }
            total
        }
        _ => return err(format!("bad time '{s}' (use 90, 1:30 or 01:02:03.5)")),
    };
    if secs < 0.0 {
        return err(format!("time must be >= 0, got '{s}'"));
    }
    Ok(secs)
}

fn secs_arg(secs: f64) -> String {
    format!("{secs:.3}")
}

fn ext_of(path: &str) -> String {
    Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default()
}

/// Video+audio codec defaults for an output path.
fn default_codecs(output: &str) -> Vec<String> {
    let h264: &[&str] = &["-c:v", "libx264", "-preset", "veryfast", "-crf", "20", "-pix_fmt", "yuv420p", "-c:a", "aac"];
    let v: Vec<&str> = match ext_of(output).as_str() {
        "mp4" | "mov" | "m4v" => {
            let mut v = h264.to_vec();
            v.extend(["-movflags", "+faststart"]);
            v
        }
        "webm" => vec![
            "-c:v", "libvpx-vp9", "-crf", "32", "-b:v", "0", "-pix_fmt", "yuv420p", "-c:a", "libopus",
        ],
        _ => h264.to_vec(),
    };
    v.iter().map(|s| s.to_string()).collect()
}

/// Audio-only codec defaults for `ll video audio`.
fn audio_codecs(output: &str) -> Vec<String> {
    match ext_of(output).as_str() {
        "mp3" => vec!["-c:a", "libmp3lame", "-q:a", "2"],
        "m4a" | "aac" => vec!["-c:a", "aac", "-b:a", "192k"],
        "wav" => vec!["-c:a", "pcm_s16le"],
        "flac" => vec!["-c:a", "flac"],
        "opus" | "ogg" => vec!["-c:a", "libopus", "-b:a", "128k"],
        _ => vec!["-c:a", "copy"],
    }
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Replace a key/value pair in an ffmpeg argument list.
fn set_pair(args: &mut Vec<String>, key: &str, value: &str) {
    if let Some(i) = args.iter().position(|x| x == key) {
        args.drain(i..=i + 1);
    }
    args.push(key.to_string());
    args.push(value.to_string());
}

/// Codecs for a re-encoding command, with --crf/--preset overrides.
fn codec_args(output: &str, o: &Opts) -> Vec<String> {
    let mut a = default_codecs(output);
    if let Some(crf) = o.get(&["--crf"]) {
        set_pair(&mut a, "-crf", crf);
    }
    if let Some(preset) = o.get(&["--preset"]) {
        set_pair(&mut a, "-preset", preset);
    }
    a
}

/// Output options for still-image outputs (screenshot/frames).
fn image_quality_args(output: &str, o: &Opts) -> Vec<String> {
    let q = o.get(&["--quality", "-q"]);
    match ext_of(output).as_str() {
        "jpg" | "jpeg" => vec!["-q:v".into(), q.unwrap_or("2").into()],
        "webp" => vec!["-quality".into(), q.unwrap_or("85").into()],
        _ => vec![],
    }
}

fn ensure_parent(path: &str) -> io::Result<()> {
    if let Some(parent) = Path::new(path).parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| {
            io::Error::new(e.kind(), format!("cannot create directory '{}': {e}", parent.display()))
        })?;
    }
    Ok(())
}

// ─── screenshot / frames ─────────────────────────────────────────────────────

const SHOT_VALS: &[&str] = &["--time", "-t", "--quality", "-q"];

fn build_screenshot(input: &str, output: &str, o: &Opts) -> io::Result<Vec<String>> {
    let mut a = Vec::new();
    if let Some(t) = arg_or(o, &["--time", "-t"], 2) {
        let t = parse_time(t)?;
        if t > 0.0 {
            a.push("-ss".into());
            a.push(secs_arg(t));
        }
    }
    a.push("-i".into());
    a.push(input.to_string());
    a.push("-frames:v".into());
    a.push("1".into());
    a.extend(image_quality_args(output, o));
    a.push(output.to_string());
    Ok(a)
}

fn cmd_screenshot(args: &[String]) -> io::Result<()> {
    let o = parse_opts(args, SHOT_VALS, &["--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file (png/jpg/webp)")?;
    ensure_parent(output)?;
    let a = build_screenshot(input, output, &o)?;
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output}");
    Ok(())
}

const FRAMES_VALS: &[&str] = &[
    "--fps", "-s", "--every", "-n", "--start", "-b", "--duration", "-d", "--format", "-f", "--quality", "-q",
];

fn build_frames(input: &str, prefix: &str, o: &Opts) -> io::Result<Vec<String>> {
    let ext = match o.get(&["--format", "-f"]) {
        Some(f) => {
            let f = normalize(f);
            if !["png", "jpg", "jpeg", "webp", "bmp"].contains(&f.as_str()) {
                return err(format!("frames: unsupported --format '{f}' (png, jpg, webp, bmp)"));
            }
            if f == "jpeg" { "jpg".to_string() } else { f }
        }
        None => "png".into(),
    };
    let pattern = format!("{prefix}-%05d.{ext}");

    let mut a = Vec::new();
    if let Some(start) = o.get(&["--start", "-b"]) {
        a.push("-ss".into());
        a.push(secs_arg(parse_time(start)?));
    }
    a.push("-i".into());
    a.push(input.to_string());
    if let Some(dur) = o.get(&["--duration", "-d"]) {
        a.push("-t".into());
        a.push(secs_arg(parse_time(dur)?));
    }

    if let Some(every) = arg_or(o, &["--every", "-n"], 2) {
        let n = parse_u32(every, "--every")?;
        if n == 0 {
            return err("frames: --every must be >= 1");
        }
        // keep frame n of every n, and renumber the output densely
        a.push("-vf".into());
        a.push(format!("select=not(mod(n\\,{n})),setpts=N/FRAME_RATE/TB"));
        a.push("-vsync".into());
        a.push("0".into());
    } else {
        let fps = arg_or(o, &["--fps", "-s"], 2).unwrap_or("1");
        let fps = parse_f64(fps, "--fps")?;
        if fps <= 0.0 {
            return err("frames: --fps must be > 0");
        }
        a.push("-vf".into());
        a.push(format!("fps={fps}"));
    }

    a.extend(image_quality_args(&pattern, o));
    a.push("-start_number".into());
    a.push("0".into());
    a.push(pattern);
    Ok(a)
}

fn cmd_frames(args: &[String]) -> io::Result<()> {
    let o = parse_opts(args, FRAMES_VALS, &["--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let prefix = pos(&o, 1, "output prefix (e.g. shots/frame)")?;
    ensure_parent(prefix)?;
    let a = build_frames(input, prefix, &o)?;
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {prefix}-NNNN.<ext>");
    Ok(())
}

// ─── trim / concat / reverse ─────────────────────────────────────────────────

const TRIM_VALS: &[&str] = &["--start", "-s", "--end", "-e", "--duration", "-d", "--crf", "--preset"];

/// `-s START -e END` / `-s START -d DUR` become ffmpeg's `-ss` + `-t`.
fn trim_window(o: &Opts) -> io::Result<(f64, f64)> {
    let start = match o.get(&["--start", "-s"]) {
        Some(v) => parse_time(v)?,
        None => 0.0,
    };
    let duration = match (o.get(&["--end", "-e"]), o.get(&["--duration", "-d"])) {
        (Some(end), None) => parse_time(end)? - start,
        (None, Some(d)) => parse_time(d)?,
        (Some(_), Some(_)) => return err("trim: use either --end or --duration, not both"),
        (None, None) => return err("trim needs --start/--end or --duration (e.g. -s 10 -e 25)"),
    };
    if duration <= 0.0 {
        return err("trim: end must be after start");
    }
    Ok((start, duration))
}

fn build_trim(input: &str, output: &str, o: &Opts) -> io::Result<Vec<String>> {
    let (start, duration) = trim_window(o)?;
    let mut a = Vec::new();
    if start > 0.0 {
        a.push("-ss".into());
        a.push(secs_arg(start));
    }
    a.push("-i".into());
    a.push(input.to_string());
    a.push("-t".into());
    a.push(secs_arg(duration));
    if o.has(&["--reencode"]) {
        a.extend(codec_args(output, o));
    } else {
        a.push("-c".into());
        a.push("copy".into());
    }
    a.push(output.to_string());
    Ok(a)
}

fn cmd_trim(args: &[String]) -> io::Result<()> {
    let vals = merge(TRIM_VALS, &["--crf", "--preset"]);
    let o = parse_opts(args, &vals, &["--reencode", "--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file")?;
    ensure_parent(output)?;
    let a = build_trim(input, output, &o)?;
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    let (start, duration) = trim_window(&o)?;
    println!("{input} [{start:.2}s +{duration:.2}s] -> {output}");
    Ok(())
}

fn concat_list_content(inputs: &[String]) -> io::Result<String> {
    if inputs.len() < 2 {
        return err("concat: give at least two input files");
    }
    let mut s = String::new();
    for i in inputs {
        let abs: PathBuf = fs::canonicalize(i).unwrap_or_else(|_| PathBuf::from(i));
        s.push_str(&format!("file '{}'\n", abs.to_string_lossy().replace('\'', "'\\''")));
    }
    Ok(s)
}

fn cmd_concat(args: &[String]) -> io::Result<()> {
    let o = parse_opts(args, &["--crf", "--preset"], &["--reencode", "--dry-run"])?;
    let output = pos(&o, 0, "output file")?;
    let inputs: Vec<String> = o.pos[1..].to_vec();
    let content = concat_list_content(&inputs)?;

    let list = std::env::temp_dir().join(format!("ll-concat-{}.txt", std::process::id()));
    fs::write(&list, content)
        .map_err(|e| io::Error::new(e.kind(), format!("cannot write '{}': {e}", list.display())))?;
    ensure_parent(output)?;

    let mut a = vec!["-f".into(), "concat".into(), "-safe".into(), "0".into()];
    a.push("-i".into());
    a.push(list.to_string_lossy().into_owned());
    if o.has(&["--reencode"]) {
        a.extend(codec_args(output, &o));
    } else {
        a.push("-c".into());
        a.push("copy".into());
    }
    a.push(output.to_string());

    let dry = o.has(&["--dry-run"]);
    let result = run_ffmpeg(&a, dry);
    let _ = fs::remove_file(&list);
    result?;
    println!("{} clips -> {output}", inputs.len());
    Ok(())
}

fn cmd_reverse(args: &[String]) -> io::Result<()> {
    let o = parse_opts(args, &["--crf", "--preset"], &["--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file")?;
    ensure_parent(output)?;
    let audio = probe_has_audio(input)?;
    let mut a = vec!["-i".into(), input.to_string()];
    a.push("-vf".into());
    a.push("reverse".into());
    if audio {
        a.push("-af".into());
        a.push("areverse".into());
    }
    a.extend(codec_args(output, &o));
    a.push(output.to_string());
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output} (reversed)");
    Ok(())
}

// ─── audio ───────────────────────────────────────────────────────────────────

fn cmd_audio(args: &[String]) -> io::Result<()> {
    let o = parse_opts(args, &[], &["--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "audio file (mp3/m4a/wav/flac/opus)")?;
    ensure_parent(output)?;
    let mut a = vec!["-i".into(), input.to_string(), "-vn".into()];
    a.extend(audio_codecs(output));
    a.push(output.to_string());
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output}");
    Ok(())
}

fn cmd_mute(args: &[String]) -> io::Result<()> {
    let o = parse_opts(args, &[], &["--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file")?;
    ensure_parent(output)?;
    let a = vec![
        "-i".into(),
        input.to_string(),
        "-c".into(),
        "copy".into(),
        "-an".into(),
        output.to_string(),
    ];
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output} (no audio)");
    Ok(())
}

/// `0.5` -> `volume=0.5`, `-6dB` -> `volume=-6dB`.
fn volume_filter(value: &str) -> io::Result<String> {
    let v = value.trim();
    if let Some(db) = v.strip_suffix("dB").or_else(|| v.strip_suffix("db")) {
        let n = parse_f64(db, "volume")?;
        return Ok(format!("volume={n}dB"));
    }
    let n = parse_f64(v, "volume")?;
    if n < 0.0 {
        return err(format!("volume must be >= 0 (use -6dB to attenuate), got '{value}'"));
    }
    Ok(format!("volume={n}"))
}

fn cmd_volume(args: &[String]) -> io::Result<()> {
    let o = parse_opts(args, &["--value", "-v"], &["--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file")?;
    let value = arg_or(&o, &["--value", "-v"], 2)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "volume needs -v FACTOR|dB (e.g. -v 0.5)"))?;
    let filter = volume_filter(value)?;
    ensure_parent(output)?;
    let mut a = vec!["-i".into(), input.to_string()];
    a.push("-c:v".into());
    a.push("copy".into());
    a.push("-af".into());
    a.push(filter);
    a.push(output.to_string());
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output} ({value})");
    Ok(())
}

// ─── transforms ──────────────────────────────────────────────────────────────

const XFORM_VALS: &[&str] = &["--crf", "--preset"];

fn build_resize(input: &str, output: &str, o: &Opts) -> io::Result<Vec<String>> {
    let keep = o.has(&["--keep-aspect", "--fit"]);
    let filter = if let Some(p) = arg_or(o, &["--percent", "-p"], 2) {
        let pct = parse_f64(p, "--percent")?;
        if pct <= 0.0 {
            return err(format!("resize: --percent must be > 0, got '{p}'"));
        }
        let k = pct / 100.0;
        format!("scale=trunc(iw*{k}/2)*2:trunc(ih*{k}/2)*2")
    } else {
        match (
            o.get(&["--width", "-w"]),
            o.get(&["--height", "-h"]),
            arg_or(o, &["--size", "-s"], 2),
        ) {
            (None, None, None) => {
                return err("resize needs -s WxH, -w WIDTH, -h HEIGHT or --percent N");
            }
            (_, _, Some(size)) => scale_geometry(size, keep)?,
            (w, h, None) => {
                let size = format!("{}x{}", w.unwrap_or("_"), h.unwrap_or("_"));
                scale_geometry(&size, keep)?
            }
        }
    };
    let mut a = vec!["-i".into(), input.to_string(), "-vf".into(), filter];
    a.extend(codec_args(output, o));
    a.push(output.to_string());
    Ok(a)
}

/// `1280x720`, `1280x_`, `_720` and `640` become a `scale=` filter.
fn scale_geometry(size: &str, keep_aspect: bool) -> io::Result<String> {
    let (w, h) = match size.trim().split_once('x') {
        Some((w, h)) => (w.to_string(), h.to_string()),
        None => (size.trim().to_string(), String::new()),
    };
    let dim = |tok: &str, what: &str| -> io::Result<Option<u32>> {
        if tok.is_empty() || tok == "_" {
            return Ok(None);
        }
        let v = parse_u32(tok, what)?;
        if v == 0 {
            return err(format!("{what} must be > 0"));
        }
        Ok(Some(v))
    };
    let (w, h) = (dim(&w, "width")?, dim(&h, "height")?);
    match (w, h) {
        (Some(w), Some(h)) if keep_aspect => Ok(format!(
            "scale={w}:{h}:force_original_aspect_ratio=decrease:force_divisible_by=2"
        )),
        (Some(w), Some(h)) => Ok(format!("scale={w}:{h}")),
        (Some(w), None) => Ok(format!("scale={w}:-2")),
        (None, Some(h)) => Ok(format!("scale=-2:{h}")),
        (None, None) => err("resize: empty size"),
    }
}

fn cmd_resize(args: &[String]) -> io::Result<()> {
    let vals = merge(&["--width", "-w", "--height", "-h", "--size", "-s", "--percent", "-p"], XFORM_VALS);
    let o = parse_opts(args, &vals, &["--keep-aspect", "--fit", "--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file")?;
    ensure_parent(output)?;
    let a = build_resize(input, output, &o)?;
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output}");
    Ok(())
}

fn parse_crop_geometry(spec: &str) -> io::Result<(u32, u32, u32, u32)> {
    let s = spec.trim().replacen('X', "x", 1);
    let Some(ix) = s.find('x') else {
        return err(format!("crop geometry must look like WxH+X+Y, got '{spec}'"));
    };
    let w_tok = &s[..ix];
    let rest = &s[ix + 1..];
    let ih = rest.find(['+', '-']).unwrap_or(rest.len());
    let h_tok = &rest[..ih];
    let tail = &rest[ih..];

    let w = parse_u32(w_tok, "crop width")?.max(1);
    let h = parse_u32(h_tok, "crop height")?.max(1);
    let offset = |tok: &str| -> io::Result<u32> {
        if tok.starts_with('-') && !tok[1..].trim_start_matches('0').is_empty() {
            return err(
                "crop: negative offsets are not supported for video — use WxH+X+Y with X/Y >= 0",
            );
        }
        let t = tok.trim_start_matches(['+', '-']);
        if t.is_empty() {
            return Ok(0);
        }
        parse_u32(t, "crop offset")
    };

    // split the tail into at most two signed numbers
    let mut offs: Vec<String> = Vec::new();
    let mut rest = tail.trim();
    while !rest.is_empty() {
        let (sign, body) = match rest.strip_prefix('+') {
            Some(b) => ('+', b),
            None => match rest.strip_prefix('-') {
                Some(b) => ('-', b),
                None => return err(format!("bad crop offsets in '{spec}'")),
            },
        };
        let end = body.find(['+', '-']).unwrap_or(body.len());
        offs.push(format!("{sign}{}", &body[..end]));
        rest = body[end..].trim();
        if offs.len() > 2 {
            return err(format!("too many offsets in crop geometry '{spec}'"));
        }
    }
    let x = match offs.first() {
        Some(t) => offset(t)?,
        None => 0,
    };
    let y = match offs.get(1) {
        Some(t) => offset(t)?,
        None => 0,
    };
    Ok((w, h, x, y))
}

fn build_crop(input: &str, output: &str, o: &Opts) -> io::Result<Vec<String>> {
    let geo = arg_or(o, &["--geometry", "-g"], 2)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "crop needs -g WxH+X+Y"))?;
    let (w, h, x, y) = parse_crop_geometry(geo)?;
    let mut a = vec![
        "-i".into(),
        input.to_string(),
        "-vf".into(),
        format!("crop={w}:{h}:{x}:{y}"),
    ];
    a.extend(codec_args(output, o));
    a.push(output.to_string());
    Ok(a)
}

fn cmd_crop(args: &[String]) -> io::Result<()> {
    let vals = merge(&["--geometry", "-g"], XFORM_VALS);
    let o = parse_opts(args, &vals, &["--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file")?;
    ensure_parent(output)?;
    let a = build_crop(input, output, &o)?;
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output}");
    Ok(())
}

fn cmd_rotate(args: &[String]) -> io::Result<()> {
    let vals = merge(&["--angle", "-a"], XFORM_VALS);
    let o = parse_opts(args, &vals, &["--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file")?;
    let angle = arg_or(&o, &["--angle", "-a"], 2)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "rotate needs -a 90|180|270"))?;
    let filter = rotate_filter(angle)?;
    ensure_parent(output)?;
    let mut a = vec!["-i".into(), input.to_string(), "-vf".into(), filter];
    a.extend(codec_args(output, &o));
    a.push(output.to_string());
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output} (rotated {angle}°)");
    Ok(())
}

fn rotate_filter(angle: &str) -> io::Result<String> {
    match parse_f64(angle, "angle")?.rem_euclid(360.0).round() as i64 {
        90 => Ok("transpose=1".into()),           // 90° clockwise
        180 => Ok("hflip,vflip".into()),
        270 => Ok("transpose=2".into()),          // 90° counter-clockwise
        0 => Ok("null".into()),
        other => err(format!("rotate: unsupported angle '{other}' (90, 180 or 270)")),
    }
}

fn cmd_flip(args: &[String]) -> io::Result<()> {
    let o = parse_opts(args, XFORM_VALS, &["--horizontal", "--vertical", "--both", "--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file")?;
    let filter = if o.has(&["--horizontal"]) && o.has(&["--vertical"]) {
        "hflip,vflip"
    } else if o.has(&["--horizontal"]) {
        "hflip"
    } else if o.has(&["--vertical"]) {
        "vflip"
    } else {
        match arg_or(&o, &["--direction", "-d"], 2) {
            Some(d) => match normalize(d).as_str() {
                "h" | "horizontal" => "hflip",
                "v" | "vertical" => "vflip",
                "both" | "hv" => "hflip,vflip",
                other => return err(format!("flip: unknown direction '{other}' (h|v|both)")),
            },
            None => return err("flip needs --horizontal and/or --vertical (or -d h|v|both)"),
        }
    };
    ensure_parent(output)?;
    let mut a = vec!["-i".into(), input.to_string(), "-vf".into(), filter.to_string()];
    a.extend(codec_args(output, &o));
    a.push(output.to_string());
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output}");
    Ok(())
}

/// ffmpeg's atempo accepts 0.5..100, so slower speeds chain filters.
fn atempo_chain(factor: f64) -> io::Result<String> {
    if factor <= 0.0 {
        return err(format!("speed must be > 0, got {factor}"));
    }
    let mut parts = Vec::new();
    let mut f = factor;
    while f < 0.5 {
        parts.push(0.5f64);
        f /= 0.5;
    }
    while f > 2.0 {
        parts.push(2.0f64);
        f /= 2.0;
    }
    parts.push(f);
    Ok(parts
        .iter()
        .map(|v| format!("atempo={v}"))
        .collect::<Vec<_>>()
        .join(","))
}

fn build_speed(input: &str, output: &str, factor: f64, audio: bool, o: &Opts) -> io::Result<Vec<String>> {
    let mut a = vec!["-i".into(), input.to_string()];
    a.push("-vf".into());
    a.push(format!("setpts=PTS/{factor}"));
    if audio {
        a.push("-af".into());
        a.push(atempo_chain(factor)?);
    }
    a.extend(codec_args(output, o));
    a.push(output.to_string());
    Ok(a)
}

fn cmd_speed(args: &[String]) -> io::Result<()> {
    let vals = merge(&["--value", "-v"], XFORM_VALS);
    let o = parse_opts(args, &vals, &["--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file")?;
    let value = arg_or(&o, &["--value", "-v"], 2)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "speed needs -v FACTOR (e.g. -v 2)"))?;
    let factor = parse_f64(value, "speed")?;
    let audio = probe_has_audio(input)?;
    ensure_parent(output)?;
    let a = build_speed(input, output, factor, audio, &o)?;
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output} ({factor}x)");
    Ok(())
}

// ─── convert / gif / sprite ──────────────────────────────────────────────────

fn codec_by_name(name: &str, output: &str) -> io::Result<Vec<String>> {
    let audio = match ext_of(output).as_str() {
        "webm" => vec!["-c:a", "libopus", "-b:a", "128k"],
        _ => vec!["-c:a", "aac"],
    };
    let mut a: Vec<String> = match normalize(name).as_str() {
        "copy" => vec!["-c", "copy"],
        "h264" | "x264" | "avc" => vec!["-c:v", "libx264", "-preset", "veryfast", "-crf", "20", "-pix_fmt", "yuv420p"],
        "hevc" | "x265" | "h265" => vec!["-c:v", "libx265", "-preset", "veryfast", "-crf", "24", "-pix_fmt", "yuv420p"],
        "vp9" | "vp09" => vec!["-c:v", "libvpx-vp9", "-crf", "32", "-b:v", "0", "-pix_fmt", "yuv420p"],
        "av1" | "aom" => vec!["-c:v", "libaom-av1", "-crf", "32", "-b:v", "0", "-pix_fmt", "yuv420p"],
        other => return err(format!("convert: unknown --codec '{other}' (h264, hevc, vp9, av1, copy)")),
    }
    .iter()
    .map(|s| s.to_string())
    .collect();
    if normalize(name) != *"copy" {
        a.extend(audio.iter().map(|s| s.to_string()));
    }
    Ok(a)
}

fn build_convert(input: &str, output: &str, o: &Opts) -> io::Result<Vec<String>> {
    let mut a = vec!["-i".into(), input.to_string()];
    let copy_mode = o.get(&["--codec", "-c"]).is_some_and(|c| normalize(c) == "copy");
    let mut filters: Vec<String> = Vec::new();
    if let Some(fps) = o.get(&["--fps", "-r"]) {
        let fps = parse_f64(fps, "--fps")?;
        if fps <= 0.0 {
            return err("convert: --fps must be > 0");
        }
        filters.push(format!("fps={fps}"));
    }
    let size = arg_or(o, &["--size", "-s"], 2).map(str::to_string).or_else(|| {
        match (o.get(&["--width", "-w"]), o.get(&["--height", "-h"])) {
            (None, None) => None,
            (w, h) => Some(format!("{}x{}", w.unwrap_or("_"), h.unwrap_or("_"))),
        }
    });
    if let Some(size) = size {
        filters.push(scale_geometry(&size, o.has(&["--keep-aspect"]))?);
    }
    if !filters.is_empty() {
        a.push("-vf".into());
        a.push(filters.join(","));
    }

    match o.get(&["--codec", "-c"]) {
        Some(c) => a.extend(codec_by_name(c, output)?),
        None => a.extend(default_codecs(output)),
    }
    if !copy_mode {
        if let Some(crf) = o.get(&["--crf"]) {
            set_pair(&mut a, "-crf", crf);
        }
        if let Some(preset) = o.get(&["--preset"]) {
            set_pair(&mut a, "-preset", preset);
        }
    }
    a.push(output.to_string());
    Ok(a)
}

fn cmd_convert(args: &[String]) -> io::Result<()> {
    let vals = merge(&["--codec", "-c", "--crf", "--preset", "--fps", "-r", "--size", "-s", "--width", "-w", "--height", "-h"], &[]);
    let o = parse_opts(args, &vals, &["--keep-aspect", "--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output file")?;
    ensure_parent(output)?;
    let a = build_convert(input, output, &o)?;
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output}");
    Ok(())
}

const GIF_VALS: &[&str] = &["--fps", "-s", "--width", "-w"];

fn build_gif(input: &str, output: &str, o: &Opts) -> io::Result<Vec<String>> {
    let fps = arg_or(o, &["--fps", "-s"], 2).unwrap_or("12");
    let fps = parse_f64(fps, "--fps")?;
    if fps <= 0.0 {
        return err("gif: --fps must be > 0");
    }
    let width = arg_or(o, &["--width", "-w"], 2).unwrap_or("480");
    let width = parse_u32(width, "--width")?;
    if width == 0 {
        return err("gif: --width must be > 0");
    }
    let chain = format!(
        "fps={fps},scale={width}:-2:flags=lanczos,split[a][b];[a]palettegen=max_colors=128[p];[b][p]paletteuse=dither=bayer:bayer_scale=5"
    );
    Ok(vec![
        "-i".into(),
        input.to_string(),
        "-loop".into(),
        "0".into(),
        "-vf".into(),
        chain,
        output.to_string(),
    ])
}

fn cmd_gif(args: &[String]) -> io::Result<()> {
    let o = parse_opts(args, GIF_VALS, &["--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output gif")?;
    ensure_parent(output)?;
    let a = build_gif(input, output, &o)?;
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output}");
    Ok(())
}

fn cmd_gif2video(args: &[String]) -> io::Result<()> {
    let vals = merge(&["--fps", "-s"], &["--crf", "--preset"]);
    let o = parse_opts(args, &vals, &["--dry-run"])?;
    let input = pos(&o, 0, "input gif")?;
    let output = pos(&o, 1, "output file (mp4/webm)")?;
    ensure_parent(output)?;
    let mut a = vec!["-i".into(), input.to_string()];
    if let Some(fps) = arg_or(&o, &["--fps", "-s"], 2) {
        a.push("-r".into());
        a.push(fps.to_string());
    }
    a.extend(codec_args(output, &o));
    a.push(output.to_string());
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output}");
    Ok(())
}

/// Pick an interval so that `cols * rows` thumbnails cover the whole clip.
fn sprite_interval(duration: Option<f64>, cols: u32, rows: u32) -> f64 {
    match duration {
        Some(d) if d > 0.0 => (d / (cols as f64 * rows as f64)).max(0.1),
        _ => 10.0,
    }
}

fn build_sprite(input: &str, output: &str, duration: Option<f64>, o: &Opts) -> io::Result<Vec<String>> {
    let cols = parse_u32(arg_or(o, &["--cols"], 2).unwrap_or("4"), "--cols")?.max(1);
    let rows = parse_u32(arg_or(o, &["--rows"], 3).unwrap_or("3"), "--rows")?.max(1);
    let width = parse_u32(o.get(&["--width", "-w"]).unwrap_or("320"), "--width")?.max(16);
    let interval = match o.get(&["--interval", "-i"]) {
        Some(v) => parse_f64(v, "--interval")?,
        None => sprite_interval(duration, cols, rows),
    };
    if interval <= 0.0 {
        return err("sprite: --interval must be > 0");
    }
    let vf = format!(
        "fps=1/{interval:.3},scale={width}:-2,tile={cols}x{rows}"
    );
    Ok(vec![
        "-i".into(),
        input.to_string(),
        "-frames:v".into(),
        "1".into(),
        "-q:v".into(),
        "2".into(),
        "-vf".into(),
        vf,
        output.to_string(),
    ])
}

fn cmd_sprite(args: &[String]) -> io::Result<()> {
    let vals = merge(&["--cols", "--rows", "--interval", "-i", "--width", "-w"], &[]);
    let o = parse_opts(args, &vals, &["--dry-run"])?;
    let input = pos(&o, 0, "input file")?;
    let output = pos(&o, 1, "output image (png/jpg)")?;
    let duration = probe_duration(input)?;
    ensure_parent(output)?;
    let a = build_sprite(input, output, duration, &o)?;
    run_ffmpeg(&a, o.has(&["--dry-run"]))?;
    println!("{input} -> {output}");
    Ok(())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(v: &[&str]) -> Opts {
        let args: Vec<String> = v.iter().map(|s| s.to_string()).collect();
        let value_flags = [
            "--time", "-t", "--quality", "-q", "--fps", "-s", "--every", "-n", "--start", "-b",
            "--duration", "-d", "--format", "-f", "--end", "-e", "--crf", "--preset", "--width", "-w",
            "--height", "-h", "--size", "--percent", "-p", "--geometry", "-g", "--angle", "-a",
            "--direction", "--value", "-v", "--codec", "-c", "--cols", "--rows", "--interval", "-i",
            "--max-width", "--max-height",
        ];
        let bool_flags = [
            "--reencode", "--keep-aspect", "--fit", "--dry-run", "--horizontal", "--vertical", "--both", "--json",
        ];
        parse_opts(&args, &value_flags, &bool_flags).unwrap()
    }

    #[test]
    fn parses_times() {
        assert_eq!(parse_time("90").unwrap(), 90.0);
        assert_eq!(parse_time("1:30").unwrap(), 90.0);
        assert_eq!(parse_time("01:02:03.5").unwrap(), 3723.5);
        assert_eq!(parse_time("0:00.5").unwrap(), 0.5);
        assert!(parse_time("-5").is_err());
        assert!(parse_time("a:b").is_err());
        assert!(parse_time("1:2:3:4").is_err());
    }

    #[test]
    fn builds_screenshots() {
        let o = opts(&["in.mp4", "shot.png", "-t", "1:30"]);
        assert_eq!(
            build_screenshot("in.mp4", "shot.png", &o).unwrap(),
            vec!["-ss", "90.000", "-i", "in.mp4", "-frames:v", "1", "shot.png"]
        );
        // jpg output carries a quality flag, default -q:v 2
        let o = opts(&["in.mp4", "shot.jpg"]);
        let a = build_screenshot("in.mp4", "shot.jpg", &o).unwrap();
        assert!(a.contains(&"-q:v".to_string()));
        assert_eq!(a[a.iter().position(|x| x == "-q:v").unwrap() + 1], "2");
    }

    #[test]
    fn builds_frame_sequences() {
        let o = opts(&["in.mp4", "shots/f", "--fps", "2.5"]);
        let a = build_frames("in.mp4", "shots/f", &o).unwrap();
        assert!(a.contains(&"fps=2.5".to_string()));
        assert!(a.contains(&"shots/f-%05d.png".to_string()));

        // --every keeps one frame out of N and renumbers densely
        let o = opts(&["in.mp4", "f", "--every", "30"]);
        let a = build_frames("in.mp4", "f", &o).unwrap();
        let vf = &a[a.iter().position(|x| x == "-vf").unwrap() + 1];
        assert!(vf.starts_with("select=not(mod(n\\,30))"));
        assert_eq!(a[a.iter().position(|x| x == "-start_number").unwrap() + 1], "0");

        let o = opts(&["in.mp4", "f", "--format", "jpeg"]);
        let a = build_frames("in.mp4", "f", &o).unwrap();
        assert!(a.contains(&"f-%05d.jpg".to_string()));

        let o = opts(&["in.mp4", "f", "--format", "tiff"]);
        assert!(build_frames("in.mp4", "f", &o).is_err());
    }

    #[test]
    fn builds_trims() {
        let o = opts(&["in.mp4", "out.mp4", "-s", "10", "-e", "25"]);
        assert_eq!(
            build_trim("in.mp4", "out.mp4", &o).unwrap(),
            vec!["-ss", "10.000", "-i", "in.mp4", "-t", "15.000", "-c", "copy", "out.mp4"]
        );
        // --reencode swaps stream copy for the codec defaults
        let o = opts(&["in.mp4", "out.mp4", "-d", "5", "--reencode"]);
        let a = build_trim("in.mp4", "out.mp4", &o).unwrap();
        assert!(!a.contains(&"copy".to_string()));
        assert!(a.contains(&"libx264".to_string()));

        let o = opts(&["in.mp4", "out.mp4", "-e", "5", "-d", "5"]);
        assert!(build_trim("in.mp4", "out.mp4", &o).is_err());
        let o = opts(&["in.mp4", "out.mp4"]);
        assert!(build_trim("in.mp4", "out.mp4", &o).is_err());
        let o = opts(&["in.mp4", "out.mp4", "-s", "20", "-e", "10"]);
        assert!(build_trim("in.mp4", "out.mp4", &o).is_err());
    }

    #[test]
    fn writes_concat_lists() {
        let content = concat_list_content(&["a.mp4".into(), "b.mp4".into()]).unwrap();
        assert!(content.lines().count() == 2);
        assert!(content.starts_with("file '"));
        assert!(concat_list_content(&["only.mp4".into()]).is_err());
    }

    #[test]
    fn builds_resizes() {
        let o = opts(&["in.mp4", "out.mp4", "-w", "1280"]);
        let a = build_resize("in.mp4", "out.mp4", &o).unwrap();
        assert!(a.contains(&"scale=1280:-2".to_string()));

        let o = opts(&["in.mp4", "out.mp4", "-h", "720"]);
        let a = build_resize("in.mp4", "out.mp4", &o).unwrap();
        assert!(a.contains(&"scale=-2:720".to_string()));

        let o = opts(&["in.mp4", "out.mp4", "-s", "640x480"]);
        let a = build_resize("in.mp4", "out.mp4", &o).unwrap();
        assert!(a.contains(&"scale=640:480".to_string()));

        // keeping the aspect ratio shrinks into the box instead of stretching
        let o = opts(&["in.mp4", "out.mp4", "-s", "640x480", "--keep-aspect"]);
        let a = build_resize("in.mp4", "out.mp4", &o).unwrap();
        assert!(a[a.iter().position(|x| x == "-vf").unwrap() + 1].contains("force_original_aspect_ratio=decrease"));

        let o = opts(&["in.mp4", "out.mp4", "--percent", "50"]);
        let a = build_resize("in.mp4", "out.mp4", &o).unwrap();
        assert!(a.contains(&"scale=trunc(iw*0.5/2)*2:trunc(ih*0.5/2)*2".to_string()));

        let o = opts(&["in.mp4", "out.mp4"]);
        assert!(build_resize("in.mp4", "out.mp4", &o).is_err());
    }

    #[test]
    fn parses_crop_geometry() {
        assert_eq!(parse_crop_geometry("640x480+10+20").unwrap(), (640, 480, 10, 20));
        assert_eq!(parse_crop_geometry("640x480").unwrap(), (640, 480, 0, 0));
        let e = parse_crop_geometry("640x480-5-0").unwrap_err().to_string();
        assert!(e.contains("negative offsets"), "{e}");
        assert!(parse_crop_geometry("640").is_err());

        let o = opts(&["in.mp4", "out.mp4", "-g", "320x240+8+16"]);
        let a = build_crop("in.mp4", "out.mp4", &o).unwrap();
        assert!(a.contains(&"crop=320:240:8:16".to_string()));
    }

    #[test]
    fn maps_rotations_and_flips() {
        assert_eq!(rotate_filter("90").unwrap(), "transpose=1");
        assert_eq!(rotate_filter("180").unwrap(), "hflip,vflip");
        assert_eq!(rotate_filter("-90").unwrap(), "transpose=2");
        assert_eq!(rotate_filter("450").unwrap(), "transpose=1");
        assert!(rotate_filter("45").is_err());
    }

    #[test]
    fn chains_atempo_for_extreme_speeds() {
        assert_eq!(atempo_chain(1.0).unwrap(), "atempo=1");
        assert_eq!(atempo_chain(2.0).unwrap(), "atempo=2");
        assert_eq!(atempo_chain(0.5).unwrap(), "atempo=0.5");
        assert_eq!(atempo_chain(0.25).unwrap(), "atempo=0.5,atempo=0.5");
        assert_eq!(atempo_chain(8.0).unwrap(), "atempo=2,atempo=2,atempo=2");
        assert!(atempo_chain(0.0).is_err());

        let o = opts(&["in.mp4", "out.mp4", "-v", "2"]);
        let a = build_speed("in.mp4", "out.mp4", 2.0, true, &o).unwrap();
        assert!(a.contains(&"setpts=PTS/2".to_string()));
        assert!(a.contains(&"atempo=2".to_string()));
        // no audio stream -> no -af at all
        let a = build_speed("in.mp4", "out.mp4", 2.0, false, &o).unwrap();
        assert!(!a.contains(&"-af".to_string()));
    }

    #[test]
    fn formats_volume() {
        assert_eq!(volume_filter("0.5").unwrap(), "volume=0.5");
        assert_eq!(volume_filter("2").unwrap(), "volume=2");
        assert_eq!(volume_filter("-6dB").unwrap(), "volume=-6dB");
        assert_eq!(volume_filter("3db").unwrap(), "volume=3dB");
        assert!(volume_filter("-2").is_err());
        assert!(volume_filter("loud").is_err());
    }

    #[test]
    fn picks_codecs_per_container() {
        assert!(default_codecs("out.mp4").contains(&"libx264".to_string()));
        assert!(default_codecs("out.mp4").contains(&"+faststart".to_string()));
        assert!(default_codecs("out.webm").contains(&"libvpx-vp9".to_string()));
        assert!(default_codecs("out.webm").contains(&"libopus".to_string()));

        assert_eq!(audio_codecs("a.mp3")[1], "libmp3lame");
        assert_eq!(audio_codecs("a.wav")[1], "pcm_s16le");
        assert_eq!(audio_codecs("a.opus")[1], "libopus");
        assert_eq!(audio_codecs("a.xyz")[1], "copy");

        assert!(codec_by_name("hevc", "o.mp4").unwrap().contains(&"libx265".to_string()));
        assert!(codec_by_name("copy", "o.mkv").unwrap() == vec!["-c".to_string(), "copy".to_string()]);
        assert!(codec_by_name("prores", "o.mov").is_err());
    }

    #[test]
    fn builds_gifs_and_sheets() {
        let o = opts(&["in.mp4", "out.gif", "-s", "10", "-w", "320"]);
        let a = build_gif("in.mp4", "out.gif", &o).unwrap();
        let vf = &a[a.iter().position(|x| x == "-vf").unwrap() + 1];
        assert!(vf.starts_with("fps=10,scale=320:-2"));
        assert!(vf.contains("palettegen") && vf.contains("paletteuse"));

        assert_eq!(sprite_interval(Some(120.0), 4, 3), 10.0);
        assert_eq!(sprite_interval(Some(12.0), 4, 3), 1.0);
        assert_eq!(sprite_interval(None, 4, 3), 10.0);

        let o = opts(&["in.mp4", "sheet.png", "--cols", "2", "--rows", "2", "--interval", "5"]);
        let a = build_sprite("in.mp4", "sheet.png", Some(100.0), &o).unwrap();
        assert!(a.contains(&"fps=1/5.000,scale=320:-2,tile=2x2".to_string()));
    }

    #[test]
    fn builds_converts() {
        let o = opts(&["in.mov", "out.mp4", "--codec", "hevc", "--crf", "22", "--fps", "30"]);
        let a = build_convert("in.mov", "out.mp4", &o).unwrap();
        assert!(a.contains(&"libx265".to_string()));
        assert_eq!(a[a.iter().position(|x| x == "-crf").unwrap() + 1], "22");
        assert!(a.contains(&"fps=30".to_string()));

        let o = opts(&["in.mov", "out.mp4", "--codec", "copy"]);
        let a = build_convert("in.mov", "out.mp4", &o).unwrap();
        assert!(a.contains(&"copy".to_string()));
        assert!(!a.contains(&"aac".to_string()));

        let o = opts(&["in.mov", "out.mp4", "-s", "640x_"]);
        let a = build_convert("in.mov", "out.mp4", &o).unwrap();
        assert!(a.contains(&"scale=640:-2".to_string()));
    }

    #[test]
    fn command_table_is_consistent() {
        for c in CMDS {
            assert!(!c.usage.is_empty() && !c.desc.is_empty());
        }
        assert_eq!(find_cmd("shot").unwrap().name, "screenshot");
        assert_eq!(find_cmd("Contact-Sheet").unwrap().name, "sprite");
        assert_eq!(find_cmd("join").unwrap().name, "concat");
        assert!(find_cmd("definitely-not-a-command").is_none());
    }
}
