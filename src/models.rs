// TODO: Comments, doc comments

use std::cell::RefCell;
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::fmt::Display;
use std::rc::Rc;

use chrono::Utc;

use crate::btree::BTree;

pub type DoctorRef = Rc<RefCell<Doctor>>;
pub type PatientRef = Rc<RefCell<Patient>>;

// ---------------------------------------------------------------------------
// Timekeeping
// ---------------------------------------------------------------------------

/// How many simulated minutes pass each time the user advances the simulation.
/// 15 is small enough to watch things change and big enough that a whole day is only ~100 steps.
pub const TICK_MINUTES: u32 = 15;

/// The simulation starts at 06:00 on day 1.
/// Time is kept as "minutes since midnight of day 1": 1440 is midnight of day 2, and so on.
pub const START_TIME: u32 = 6 * 60;

/// A condition level of 10 is critical (from the project description).
pub const CRITICAL: u8 = 10;

/// Turns simulation minutes into text like "Day 1 08:15".
pub fn format_time(minutes: u32) -> String {
    let day = minutes / 1440 + 1;
    let hour = (minutes % 1440) / 60;
    let minute = minutes % 60;
    format!("Day {} {:02}:{:02}", day, hour, minute)
}

/// How many minutes one treatment takes.
///
/// A sicker patient takes longer, and a doctor who is MORE qualified for the condition
/// (a higher maximum severity they can handle, see `Doctor::conditions`) is faster:
///   base minutes = 30 + 15 * severity          (severity 10 -> 150 min)
///   real minutes = base * 10 / (5 + skill)     (skill 10 -> x0.67, skill 3 -> x1.25)
/// The result is rounded UP to a whole number of ticks, so a treatment always ends exactly on
/// a tick (the clock only moves in ticks).
/// Called by `Hospital::assign_doctor`.
pub fn treatment_minutes(severity: u8, skill: u32) -> u32 {
    let base = 30 + (severity as u32) * 15;
    let raw = base * 10 / (5 + skill);
    let ticks = (raw + TICK_MINUTES - 1) / TICK_MINUTES;
    if ticks == 0 {
        return TICK_MINUTES;
    }
    ticks * TICK_MINUTES
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PersonalInformation {
    /// This person's last name.
    last_name: String,
    /// This person's first name.
    first_name: String,
    /// UNIX time representing birth date
    date_of_birth: i64,
    /// This patient's age in years
    age: u8,
}

impl PersonalInformation {
    #[inline]
    #[must_use]
    pub fn new(first_name: impl Into<String>, last_name: impl Into<String>, date_of_birth: impl AsRef<str>) -> Self {
        let dob = format!("{} 00:00:00", date_of_birth.as_ref());
        let dob = chrono::NaiveDateTime::parse_from_str(&dob, "%Y-%m-%d %H:%M:%S").unwrap().and_utc();
        let age = Utc::now().years_since(dob).unwrap() as u8;

        Self { last_name: last_name.into(), first_name: first_name.into(), date_of_birth: dob.timestamp(), age }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Condition {
    /// The severity of the condition.
    /// Lowest is 0; highest is 10.
    pub severity: u8,
    /// The medical condition.
    pub condition: Rc<str>,
}

impl Condition {
    #[inline]
    #[must_use]
    pub fn new(condition: impl Into<Rc<str>>, severity: u8) -> Self {
        Self { condition: condition.into(), severity }
    }
}

#[derive(Debug)]
pub struct Doctor {
    /// This doctor's name.
    pub personal_information: PersonalInformation,
    /// The conditions this doctor specializes in treating, and the maximum severity they can
    /// handle.
    pub conditions: BinaryHeap<Condition>,
    /// Whether this doctor is currently available.
    pub available: bool,
    /// +the hours of the day this doctor works.
    pub shift: Shift,
}

impl Doctor {
    #[inline]
    #[must_use]
    pub fn new(personal_information: PersonalInformation, conditions: impl Into<BinaryHeap<Condition>>) -> Self {
        Self { personal_information, conditions: conditions.into(), available: true, shift: Shift::always() }
    }

    /// Sets this doctor's shift (0-24 in hrs).
    /// Without it a doctor works around the clock.
    #[must_use]
    pub fn with_shift(mut self, start: u32, end: u32) -> Self {
        self.shift = Shift::new(start, end);
        self
    }

    /// The most severe case of `condition` this doctor can treat. 0 if they don't treat it.
    /// The doctor's "skill" for that condition, it decides how fast they are.
    pub fn skill_for(&self, condition: &str) -> u32 {
        for c in self.conditions.iter() {
            if &*c.condition == condition {
                return c.severity as u32;
            }
        }
        0
    }

    pub fn slug(&self) -> String {
        format!("doctor-{}-{}-{}", self.personal_information.last_name, self.personal_information.first_name, self.personal_information.date_of_birth)
    }
}

impl PartialEq for Doctor {
    fn eq(&self, other: &Self) -> bool {
        self.personal_information == other.personal_information
    }
}

impl Eq for Doctor {}

impl PartialOrd for Doctor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Doctor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.personal_information.cmp(&other.personal_information)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PatientSlug(String);

#[derive(Debug)]
pub struct Patient {
    /// This patient's name.
    pub personal_information: PersonalInformation,
    /// The conditions afflicting this patient.
    pub conditions: BinaryHeap<Condition>,
    /// Whether the patient is currently being treated.
    pub being_treated: bool,
    /// Minute (since midnight of day 1) that the patient walks in.
    pub arrival: u32,
    /// How many minutes this patient can wait before their untreated conditions start to get
    /// worse.  (This is in the README too) Sicker patients have less time (11 - worst severity) * 180 minutes, so a
    /// patient at severity 9 has 6 hours and one at severity 2 has 27 hours.
    pub time_limit: u32,
    /// Names of the conditions a doctor is working on right now.
    pub treating: Vec<Rc<str>>,
    /// True once every condition is cured and the patient has left.
    pub discharged: bool,
    /// True if waiting made this patient reach the critical level.
    pub went_critical: bool,
}

impl Patient {
    #[inline]
    #[must_use]
    pub fn new(personal_information: PersonalInformation, conditions: impl Into<BinaryHeap<Condition>>) -> Self {
        let conditions: BinaryHeap<Condition> = conditions.into();

        // Find the worst condition to work out how long this patient can wait.
        let mut worst: u32 = 0;
        for c in conditions.iter() {
            if c.severity as u32 > worst {
                worst = c.severity as u32;
            }
        }
        let time_limit = (11 - worst) * 180;

        Self {
            personal_information,
            conditions,
            being_treated: false,
            // Unless told otherwise patients are ALREADY in the hospital when it opens.
            arrival: START_TIME,
            time_limit,
            treating: Vec::new(),
            discharged: false,
            went_critical: false,
        }
    }

    /// Makes this patient walk in `minutes` after the simulation starts.
    /// Several patients can be given the same time when they arrive together.
    #[must_use]
    pub fn arriving_at(mut self, minutes: u32) -> Self {
        self.arrival = START_TIME + minutes;
        self
    }

    /// Whether this patient is inside the hospital at time `now`.
    pub fn has_arrived(&self, now: u32) -> bool {
        self.arrival <= now
    }

    /// How many minutes this patient has been in the hospital.
    /// saturating_sub gives 0 instead of crashing when `now` is before `arrival`.
    pub fn wait_so_far(&self, now: u32) -> u32 {
        now.saturating_sub(self.arrival)
    }

    pub fn slug(&self) -> PatientSlug {
        PatientSlug(format!("patient-{}-{}-{}", self.personal_information.last_name, self.personal_information.first_name, self.personal_information.date_of_birth))
    }
}

impl PartialEq for Patient {
    fn eq(&self, other: &Self) -> bool {
        self.personal_information == other.personal_information
    }
}

impl Eq for Patient {}

impl PartialOrd for Patient {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Patient {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.personal_information.cmp(&other.personal_information)
    }
}

/// One doctor treating one condition of one patient, until `finish_time`.
/// The hospital keeps a list of these so it knows what is going on between ticks.
pub struct Treatment {
    pub doctor: DoctorRef,
    pub patient: PatientRef,
    pub condition: Condition,
    /// Minute the treatment is over.
    pub finish_time: u32,
}

pub struct Hospital {
    /// A sorted set of all doctors at this hospital.
    pub doctors: BTree<DoctorRef>,

    /// This hospital's doctors, by condition.
    pub doctors_by_condition: HashMap<Rc<str>, BTreeMap<u8, Vec<DoctorRef>>>,

    /// All patients at this hospital.
    pub patients: BTree<PatientRef>,

    /// A queue of patients to be treated, sorted first by priority, then by personal information.
    pub patient_queue: BinaryHeap<(usize, PatientRef)>,

    /// The simulation clock, in minutes since midnight of day 1.
    pub now: u32,
    /// Every treatment that is going on right now.
    pub treatments: Vec<Treatment>,
}

impl Hospital {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            doctors: BTree::new(),
            doctors_by_condition: HashMap::new(),
            patients: BTree::new(),
            patient_queue: BinaryHeap::new(),
            now: START_TIME,
            treatments: Vec::new(),
        }
    }

    /// Rebuilds the priority queue from scratch.
    /// Only patients who are already in the hospital, not discharged, and still have a
    /// condition nobody is treating yet go in.  It is rebuilt instead of kept up to date
    /// because priorities will be changing over time while the patients wait. (I thought this was clever)
    pub fn generate_patient_queue(&mut self) {
        self.patient_queue.clear();

        for patient in self.patients.inorder().iter() {
            let patient_ref = patient.borrow();

            if patient_ref.discharged || !patient_ref.has_arrived(self.now) {
                continue;
            }
            
            let priority = patient_ref.priority();
            if priority == 0 {
                continue;
            }
            self.patient_queue.push((priority, Rc::clone(patient)));
        }
    }

    /// Select the least-qualified doctor available to treat this condition.
    ///
    /// The idea in doing this is to try to leave more qualified doctors free in case a patient with
    /// more severe conditions arrives. Of course, this isn't perfect since a doctor may be highly
    /// qualified for one condition and less qualified for another, and be chosen for the less
    /// severe condition, leaving them unavailable in the event a patient with a severe case of the
    /// first condition arrives.
    ///
    /// Timekeeping Portion
    ///    a doctor must be ON SHIFT at the current time (`Doctor::on_shift`);
    ///    note: ALL qualified groups are searched from least to most qualified, the first version
    ///    only looked at the least qualified group, so a free expert was never used when the
    ///    group below them was busy. A patient can get worse than any doctor can handle while waiting, 
    ///    so the severity asked for is capped at the best doctor's level
    pub fn select_doctor(&self, condition: &Condition) -> Option<DoctorRef> {
        let groups = self.doctors_by_condition.get(&condition.condition)?;

        let best = *groups.keys().next_back()?;
        let needed = if condition.severity > best { best } else { condition.severity };

        for (_level, doctors) in groups.range(needed..) {
            for doctor in doctors {
                let d = doctor.borrow();
                if d.available && d.shift.on_duty(self.now) {
                    return Some(Rc::clone(doctor));
                }
            }
        }
        None
    }

    /// Gives `pc` of this patient to this doctor.  Returns the minute when the treatment ends.
    pub fn assign_doctor(doctor: &mut Doctor, patient: &mut Patient, pc: &Condition, now: u32) -> u32 {
        let minutes = treatment_minutes(pc.severity, doctor.skill_for(&pc.condition));
        let finish = now + minutes;

        println!("[{}] Assigning {doctor} to {patient} to treat {pc}, done at {}", format_time(now), format_time(finish));
        doctor.available = false;
        patient.being_treated = true;
        patient.treating.push(Rc::clone(&pc.condition));
        finish
    }

    pub fn mark_doctor_available(doctor: &mut Doctor, now: u32) {
        println!("[{}] {doctor} is now available", format_time(now));
        doctor.available = true;
    }

    /// The patient leaves!  They stay in the patients B-Tree (only insert and search exist)
    /// but are marked `discharged` so the rest of the simulation ignores them!
    pub fn discharge_patient(&mut self, patient: &mut Patient) {
        println!("[{}] Discharging {patient}", format_time(self.now));
        assert! { patient.conditions.is_empty() }
        patient.being_treated = false;
        patient.discharged = true;
    }

    // -----------------------------------------------------------------------
    // The simulation clock
    // -----------------------------------------------------------------------

    /// Does time 0 work when the first doctors take the first patients?
    /// Called once from `main` before the first tick.
    pub fn start(&mut self) {
        println!("[{}] Simulation started", format_time(self.now));
        self.generate_patient_queue();
        self.assign_doctors();
    }

    pub fn tick(&mut self) {
        let before = self.now;
        self.now += TICK_MINUTES;
        println!("===== {} =====", format_time(self.now));

        self.announce_arrivals(before);
        self.finish_treatments();
        self.make_waiting_patients_worse();
        self.generate_patient_queue();
        self.assign_doctors();
    }

    /// True when every patient has been discharged.
    pub fn is_finished(&self) -> bool {
        for patient in self.patients.inorder() {
            if !patient.borrow().discharged {
                return false;
            }
        }
        true
    }

    /// Prints the patients whose arrival time is in `before` now].
    fn announce_arrivals(&self, before: u32) {
        for patient in self.patients.inorder() {
            let patient = patient.borrow();
            if patient.arrival > before && patient.arrival <= self.now {
                println!("[{}] {patient} arrived (priority {})", format_time(patient.arrival), patient.priority());
            }
        }
    }

    /// Step 2 of tick: treatments whose time is over.
    fn finish_treatments(&mut self) {
        // Take the list out of `self` so now free to call other `&mut self` methods below.
        let treatments = std::mem::take(&mut self.treatments);
        let mut still_going: Vec<Treatment> = Vec::new();

        for treatment in treatments {
            if treatment.finish_time > self.now {
                still_going.push(treatment);
                continue;
            }

            // The doctor is free again.
            {
                let mut doctor = treatment.doctor.borrow_mut();
                Hospital::mark_doctor_available(&mut doctor, self.now);
            }

            let mut patient = treatment.patient.borrow_mut();
            let cured = Rc::clone(&treatment.condition.condition);

            // The cured condition is removed from the patient.
            let remaining: Vec<Condition> = patient.conditions.drain()
                .filter(|c| c.condition != cured)
                .collect();
            patient.conditions = BinaryHeap::from(remaining);
            patient.treating.retain(|name| *name != cured);

            println!("[{}] {patient} is cured of {cured}", format_time(self.now));

            if patient.conditions.is_empty() {
                self.discharge_patient(&mut patient);
            }
        }

        self.treatments = still_going;
    }

    /// Next step of tick: a patient who has been waiting longer than their time limit gets
    /// one level worse per tick on every condition nobody is treating, up to 10.
    fn make_waiting_patients_worse(&mut self) {
        for patient in self.patients.inorder() {
            let mut patient = patient.borrow_mut();

            if patient.discharged || !patient.has_arrived(self.now) {
                continue;
            }
            if patient.wait_so_far(self.now) <= patient.time_limit {
                continue;
            }

            let treating = patient.treating.clone();
            let mut list: Vec<Condition> = patient.conditions.drain().collect();
            let mut reached_critical = false;

            for c in list.iter_mut() {
                if treating.contains(&c.condition) {
                    continue; // doctor is already tending
                }
                if c.severity < CRITICAL {
                    c.severity += 1;
                    if c.severity == CRITICAL {
                        reached_critical = true;
                    }
                }
            }
            patient.conditions = BinaryHeap::from(list);

            if reached_critical && !patient.went_critical {
                patient.went_critical = true;
                println!("[{}] !! {patient} waited too long and is now CRITICAL", format_time(self.now));
            }
        }
    }

    /// Step 5 of tick: take patients from the priority queue, most urgent first, and give
    /// each of their untreated conditions to a free doctor.  If no doctor is free it stays untreated until next tick.
    fn assign_doctors(&mut self) {
        while let Some((_priority, patient_ref)) = self.patient_queue.pop() {
            let mut conditions: Vec<Condition> = patient_ref.borrow().conditions.iter().cloned().collect();
            let treating = patient_ref.borrow().treating.clone();

            // Most severe conditions first.
            conditions.sort_by(|a, b| b.severity.cmp(&a.severity));

            for condition in conditions {
                if treating.contains(&condition.condition) {
                    continue;
                }

                let doctor_ref = match self.select_doctor(&condition) {
                    Some(doctor_ref) => doctor_ref,
                    None => continue,
                };

                let finish_time;
                {
                    let mut doctor = doctor_ref.borrow_mut();
                    let mut patient = patient_ref.borrow_mut();
                    finish_time = Hospital::assign_doctor(&mut doctor, &mut patient, &condition, self.now);
                }

                self.treatments.push(Treatment {
                    doctor: doctor_ref,
                    patient: Rc::clone(&patient_ref),
                    condition,
                    finish_time,
                });
            }
        }
    }

    /// Prints a summary of the hospital at the current time.
    pub fn print_status(&self) {
        let mut discharged = 0;
        for patient in self.patients.inorder() {
            if patient.borrow().discharged {
                discharged += 1;
            }
        }

        println!("--- {} | waiting {} | in treatment {} | discharged {} of {} ---",
            format_time(self.now), self.patient_queue.len(), self.treatments.len(), discharged, self.patients.inorder().len());

        // The queue was emptied by `assign_doctors` so rebuild copy just to print it.
        let mut waiting: Vec<(usize, PatientRef)> = Vec::new();
        for patient in self.patients.inorder() {
            let p = patient.borrow();
            if !p.discharged && p.has_arrived(self.now) && p.priority() > 0 {
                waiting.push((p.priority(), Rc::clone(patient)));
            }
        }
        waiting.sort_by(|a, b| b.0.cmp(&a.0));
        for (priority, patient) in waiting.iter() {
            let p = patient.borrow();
            println!("  waiting: {} (priority {}, waited {} min, limit {})", p, priority, p.wait_so_far(self.now), p.time_limit);
        }
        for treatment in self.treatments.iter() {
            println!("  treating: {} treats {} for {} until {}",
                treatment.doctor.borrow(), treatment.patient.borrow(), treatment.condition, format_time(treatment.finish_time));
        }
    }

    pub fn add_doctor(&mut self, doctor: Doctor) {
        self.doctors.insert(Rc::new(RefCell::new(doctor)));
    }

    pub fn add_patient(&mut self, patient: Patient) {
        self.patients.insert(Rc::new(RefCell::new(patient)));
    }
}

impl Patient {
    /// An aggregate severity score representing urgency for treatment.  
    // Conditions a doctor is already working on do not count. they are being taken care of.
    pub fn priority(&self) -> usize {
        self.conditions.iter()
            .filter(|c| !self.treating.contains(&c.condition))
            .fold(0, |acc, x| acc + x.severity as usize)
    }
}

impl Display for Patient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.personal_information.first_name, self.personal_information.last_name)
    }
}

impl Display for Doctor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dr. {} {}", self.personal_information.first_name, self.personal_information.last_name)
    }
}

impl Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.condition, self.severity)
    }
}

/// The hours of the day a doctor works.
/// (The first version stored UNIX times here, but a repeating daily shift is simpler.)
#[derive(Debug)]
pub struct Shift {
    /// Hour of the day the shift starts.
    pub start: u32,
    /// Hour of the day the shift ends.
    pub end: u32,
}

impl Shift {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// A "shift" covering all 24 hours, the default for doctors without one.
    pub fn always() -> Self {
        Self { start: 0, end: 24 }
    }

    /// Is a doctor with this shift at work at simulation time `now` in minutes?
    pub fn on_duty(&self, now: u32) -> bool {
        let hour = (now % 1440) / 60;

        if self.start < self.end {
            // Avg shift for example 8 to 16.
            hour >= self.start && hour < self.end
        } else {
            // Night shift over midnight: 22 to 6.
            hour >= self.start || hour < self.end
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_shift() {
        let shift = Shift::new(8, 16);
        assert!(shift.on_duty(8 * 60));
        assert!(shift.on_duty(15 * 60 + 59));
        assert!(!shift.on_duty(16 * 60));
        assert!(!shift.on_duty(3 * 60));
    }

    #[test]
    fn night_shift_runs_over_midnight() {
        let shift = Shift::new(22, 6);
        assert!(shift.on_duty(23 * 60));
        assert!(shift.on_duty(2 * 60));
        assert!(shift.on_duty(1440 + 2 * 60)); // 02:00 on day 2
        assert!(!shift.on_duty(12 * 60));
    }

    #[test]
    fn default_shift_is_always_on() {
        let shift = Shift::always();
        assert!(shift.on_duty(0));
        assert!(shift.on_duty(23 * 60 + 59));
    }

    #[test]
    fn more_skilled_doctors_are_faster() {
        let slow = treatment_minutes(8, 3);
        let fast = treatment_minutes(8, 10);
        assert!(fast < slow);
        assert_eq!(fast % TICK_MINUTES, 0);
    }

    #[test]
    fn time_text() {
        assert_eq!(format_time(360), "Day 1 06:00");
        assert_eq!(format_time(1440 + 75), "Day 2 01:15");
    }

    #[test]
    fn sicker_patients_have_less_time() {
        let sick = Patient::new(PersonalInformation::new("A", "B", "2000-01-01"), vec![Condition::new("X", 9)]);
        let mild = Patient::new(PersonalInformation::new("C", "D", "2000-01-02"), vec![Condition::new("X", 2)]);
        assert!(sick.time_limit < mild.time_limit);
    }
}
