//! The SFML window for the simulation.
//!
//! Three panels show the hospital at the current time:
//! * Waiting: patients who are in the hospital and have a condition nobody is treating.
//! * Being treated: each doctor -> patient assignment with a progress bar.
//! * Doctors: who is free, busy, or off shift.
//!
//! Time only moves when the user asks for it (same as the terminal version).
//! This file only READS the [`Hospital`], the only thing it changes is the clock through
//! [`Hospital::tick`].

use std::cmp::Reverse;

use sfml::SfResult;
use sfml::graphics::{
    Color, Font, RectangleShape, RenderTarget, RenderWindow, Shape, Text, Transformable,
};
use sfml::system::Vector2f;
use sfml::window::{ContextSettings, Event, Key, Style};

use crate::models::{Hospital, PatientRef, TICK_MINUTES, format_time, treatment_minutes};

// ---------------------------------------------------------------------------
// Layout and colors
// ---------------------------------------------------------------------------
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

/// Space between the window edge and the panels and between panels.
const MARGIN: f32 = 20.0;
const PANEL_WIDTH: f32 = 400.0;
const PANEL_TOP: f32 = 110.0;
const PANEL_HEIGHT: f32 = 570.0;
/// Where the list rows start inside a panel (under the panel title).
const ROWS_TOP: f32 = 48.0;

/// Multiplies every text size below, so the whole window can be made bigger or smaller.
const TEXT_SCALE: f32 = 1.05;

const BACKGROUND: Color = Color::rgb(24, 28, 36);
const PANEL: Color = Color::rgb(36, 42, 54);
const BAR_BACK: Color = Color::rgb(72, 82, 100);
const TEXT: Color = Color::rgb(255, 252, 249);
const MUTED: Color = Color::rgb(170, 180, 196);
const GREEN: Color = Color::rgb(6, 214, 160);
const YELLOW: Color = Color::rgb(255, 209, 102);
const RED: Color = Color::rgb(214, 64, 69);
const BLUE: Color = Color::rgb(110, 185, 230);

/// The font is built into the program so it works from any folder.
/// Open Sans (SIL Open Font License, see assets/OFL.txt).
static FONT_DATA: &[u8] = include_bytes!("../assets/OpenSans-Regular.ttf");

// ---------------------------------------------------------------------------
// Small drawing helpers
// ---------------------------------------------------------------------------

/// Draws `string` with its top left corner at (`x`, `y`).
fn draw_text(window: &mut RenderWindow, font: &Font, string: &str, size: u32, color: Color, x: f32, y: f32) {
    let mut text = Text::new(string, font, (size as f32 * TEXT_SCALE).round() as u32);
    text.set_fill_color(color);
    text.set_position((x, y));
    window.draw(&text);
}

/// Draws a filled rectangle.
fn draw_rect(window: &mut RenderWindow, x: f32, y: f32, width: f32, height: f32, color: Color) {
    let mut rect = RectangleShape::with_size(Vector2f::new(width, height));
    rect.set_position((x, y));
    rect.set_fill_color(color);
    window.draw(&rect);
}

/// Draws a progress bar that is `fraction` (0.0 to 1.0) full.
fn draw_bar(window: &mut RenderWindow, x: f32, y: f32, width: f32, height: f32, fraction: f32, color: Color) {
    draw_rect(window, x, y, width, height, BAR_BACK);
    draw_rect(window, x, y, width * fraction.clamp(0.0, 1.0), height, color);
}

/// Draws a panel background with its title. The rows go on top of it.
fn draw_panel(window: &mut RenderWindow, font: &Font, x: f32, title: &str) {
    draw_rect(window, x, PANEL_TOP, PANEL_WIDTH, PANEL_HEIGHT, PANEL);
    draw_text(window, font, title, 22, TEXT, x + 12.0, PANEL_TOP + 8.0);
}

/// Green when the patient can wait, yellow when it is getting serious, red when it is bad.
/// The priority is the total severity of the conditions nobody is treating.
const fn priority_color(priority: usize) -> Color {
    if priority >= 15 {
        RED
    } else if priority >= 8 {
        YELLOW
    } else {
        GREEN
    }
}

// ---------------------------------------------------------------------------
// The panels
// ---------------------------------------------------------------------------

/// The patients who are waiting, most urgent first.
/// Same rule as `Hospital::print_status`, which is also how the terminal version lists them.
fn waiting_patients(hospital: &Hospital) -> Vec<(usize, PatientRef)> {
    let mut waiting: Vec<(usize, PatientRef)> = Vec::new();
    for patient in hospital.patients.inorder() {
        let p = patient.borrow();
        if p.has_arrived(hospital.now) && p.priority() > 0 {
            waiting.push((p.priority(), std::rc::Rc::clone(patient)));
        }
    }
    waiting.sort_by_key(|entry| Reverse(entry.0));
    waiting
}

/// Header: the clock and the totals.
fn draw_header(window: &mut RenderWindow, font: &Font, hospital: &Hospital, waiting: usize) {
    let mut not_arrived = 0;
    let all = hospital.patients.inorder();
    for patient in &all {
        let p = patient.borrow();
        if !p.has_arrived(hospital.now) {
            not_arrived += 1;
        }
    }

    draw_text(window, font, "Nota Hospital", 34, TEXT, MARGIN, 14.0);
    draw_text(window, font, &format_time(hospital.now), 34, BLUE, 860.0 + 12.0, 14.0);

    let stats = format!(
        "Waiting {waiting}     In treatment {in_treatment}     Discharged {patients_discharged} of {total_patients}     Not arrived yet {not_arrived}",
        in_treatment = hospital.treatments.len(),
        patients_discharged = hospital.patients_discharged,
        total_patients = hospital.total_patients
    );
    draw_text(window, font, &stats, 18, MUTED, MARGIN, 74.0);

    if hospital.is_finished() {
        draw_text(window, font, "Every patient has been discharged.", 18, GREEN, 860.0 + 12.0, 74.0);
    }
}

/// Left panel: waiting patients with their priority and how much of their time limit is used.
fn draw_waiting(window: &mut RenderWindow, font: &Font, hospital: &Hospital, waiting: &[(usize, PatientRef)]) {
    let x = MARGIN;
    draw_panel(window, font, x, &format!("Waiting ({})", waiting.len()));
    draw_text(window, font, "priority", 12, MUTED, x + 205.0, PANEL_TOP + 16.0);
    draw_text(window, font, "wait vs limit", 12, MUTED, x + 275.0, PANEL_TOP + 16.0);

    let row_height = 30.0;
    let max_rows = ((PANEL_HEIGHT - ROWS_TOP - 10.0) / row_height) as usize;

    for (row, (priority, patient)) in waiting.iter().take(max_rows).enumerate() {
        let p = patient.borrow();
        let y = PANEL_TOP + ROWS_TOP + row as f32 * row_height;

        // Patients who waited too long and reached level 10 get a red name.
        let name_color = if p.went_critical { RED } else { TEXT };
        draw_text(window, font, &p.to_string(), 16, name_color, x + 12.0, y);
        draw_text(window, font, &priority.to_string(), 16, priority_color(*priority), x + 220.0, y);

        // The bar fills up as the patient uses up their time limit. Red once it is over.
        let used = p.wait_so_far(hospital.now) as f32 / p.time_limit as f32;
        let bar_color = if used >= 1.0 { RED } else if used >= 0.5 { YELLOW } else { GREEN };
        draw_bar(window, x + 270.0, y + 9.0, 120.0, 8.0, used, bar_color);
    }

    if waiting.len() > max_rows {
        let more = format!("+ {} more", waiting.len() - max_rows);
        draw_text(window, font, &more, 14, MUTED, x + 12.0, PANEL_TOP + PANEL_HEIGHT - 28.0);
    }
}

/// Middle panel: one row per doctor -> patient assignment, with a progress bar.
fn draw_treatments(window: &mut RenderWindow, font: &Font, hospital: &Hospital) {
    let x = MARGIN + PANEL_WIDTH + MARGIN;
    draw_panel(window, font, x, &format!("Being treated ({})", hospital.treatments.len()));

    let row_height = 54.0;
    let max_rows = ((PANEL_HEIGHT - ROWS_TOP - 10.0) / row_height) as usize;

    for (row, treatment) in hospital.treatments.iter().take(max_rows).enumerate() {
        let y = PANEL_TOP + ROWS_TOP + row as f32 * row_height;
        let doctor = treatment.doctor.borrow();
        let patient = treatment.patient.borrow();

        // The treatment only stores when it ends, so work out how long it takes (the same
        // formula `Hospital::assign_doctor` used) to know how far along it is.
        let minutes = treatment_minutes(treatment.condition.severity, doctor.skill_for(&treatment.condition.condition));
        let start = treatment.finish_time.saturating_sub(minutes);
        let done = hospital.now.saturating_sub(start) as f32 / minutes as f32;

        draw_text(window, font, &format!("{doctor} -> {patient}"), 15, TEXT, x + 12.0, y);
        draw_text(window, font, &treatment.condition.to_string(), 13, MUTED, x + 12.0, y + 22.0);
        draw_text(window, font, &format_time(treatment.finish_time), 13, MUTED, x + 285.0, y + 22.0);
        draw_bar(window, x + 12.0, y + 44.0, 376.0, 5.0, done, BLUE);
    }

    if hospital.treatments.len() > max_rows {
        let more = format!("+ {} more", hospital.treatments.len() - max_rows);
        draw_text(window, font, &more, 14, MUTED, x + 12.0, PANEL_TOP + PANEL_HEIGHT - 28.0);
    }
}

/// Right panel: every doctor, with a colored square for free (green), busy (blue), or off shift (gray).
fn draw_doctors(window: &mut RenderWindow, font: &Font, hospital: &Hospital) {
    let x = 2.0 * (MARGIN + PANEL_WIDTH) + MARGIN;
    draw_panel(window, font, x, "Doctors");

    let row_height = 32.0;
    for (row, doctor) in hospital.doctors.inorder().into_iter().enumerate() {
        let d = doctor.borrow();
        let y = PANEL_TOP + ROWS_TOP + row as f32 * row_height;

        let (status, color) = if !d.shift.on_duty(hospital.now) && d.available {
            ("off shift", BAR_BACK)
        } else if d.available {
            ("free", GREEN)
        } else {
            ("busy", BLUE)
        };

        draw_rect(window, x + 12.0, y + 5.0, 12.0, 12.0, color);
        draw_text(window, font, &d.to_string(), 16, if status == "off shift" { MUTED } else { TEXT }, x + 34.0, y);
        draw_text(window, font, &format!("{:02}-{:02}", d.shift.start, d.shift.end % 24), 13, MUTED, x + 270.0, y + 3.0);
        draw_text(window, font, status, 13, color_for_text(color), x + 330.0, y + 3.0);
    }
}

/// The off shift square is too dark to read as text, so use the muted color for it.
fn color_for_text(color: Color) -> Color {
    if color == BAR_BACK { MUTED } else { color }
}

// ---------------------------------------------------------------------------
// The window loop
// ---------------------------------------------------------------------------

/// Opens the window and runs the simulation until the window is closed.
///
/// * SPACE or ENTER: one tick (15 minutes)
/// * N: one hour (four ticks)
/// * Q or ESC: close
///
/// Returns an error if the window or the font cannot be created, so the caller can fall
/// back to the terminal.
pub fn run(hospital: &mut Hospital) -> SfResult<()> {
    let font = Font::from_memory_static(FONT_DATA)?;
    let mut window = RenderWindow::new(
        [WINDOW_WIDTH, WINDOW_HEIGHT],
        "Hospital Simulation",
        Style::TITLEBAR | Style::CLOSE,
        &ContextSettings::default(),
    )?;
    window.set_framerate_limit(30);

    while window.is_open() {
        // How many ticks the user asked for this frame.
        let mut steps = 0;

        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed
                | Event::KeyPressed { code: Key::Escape | Key::Q, .. } => window.close(),
                Event::KeyPressed { code: Key::Space | Key::Enter, .. } => steps += 1,
                Event::KeyPressed { code: Key::N, .. } => steps += 60 / TICK_MINUTES,
                _ => {}
            }
        }

        for _ in 0..steps {
            if hospital.is_finished() {
                break;
            }
            hospital.tick();
        }

        let waiting = waiting_patients(hospital);

        window.clear(BACKGROUND);
        draw_header(&mut window, &font, hospital, waiting.len());
        draw_waiting(&mut window, &font, hospital, &waiting);
        draw_treatments(&mut window, &font, hospital);
        draw_doctors(&mut window, &font, hospital);
        draw_text(
            &mut window,
            &font,
            "SPACE / ENTER: next 15 minutes      N: next hour      Q / ESC: quit",
            15,
            MUTED,
            MARGIN,
            WINDOW_HEIGHT as f32 - 30.0,
        );
        window.display();
    }

    Ok(())
}
