// TODO: Comments, doc comments

use std::cell::RefCell;
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::fmt::Display;
use std::rc::Rc;

use chrono::Utc;

use crate::btree::BTree;

pub type DoctorRef = Rc<RefCell<Doctor>>;
pub type PatientRef = Rc<RefCell<Patient>>;

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
}

impl Doctor {
    #[inline]
    #[must_use]
    pub fn new(personal_information: PersonalInformation, conditions: impl Into<BinaryHeap<Condition>>) -> Self {
        Self { personal_information, conditions: conditions.into(), available: true }
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
}

impl Patient {
    #[inline]
    #[must_use]
    pub fn new(personal_information: PersonalInformation, conditions: impl Into<BinaryHeap<Condition>>) -> Self {
        Self { personal_information, conditions: conditions.into(), being_treated: false }
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

#[derive(Default)]
pub struct Hospital {
    /// A sorted set of all doctors at this hospital.
    pub doctors: BTree<DoctorRef>,

    /// This hospital's doctors, by condition.
    pub doctors_by_condition: HashMap<Rc<str>, BTreeMap<u8, Vec<DoctorRef>>>,

    /// All patients at this hospital.
    pub patients: BTree<PatientRef>,

    /// A queue of patients to be treated, sorted first by priority, then by personal information.
    pub patient_queue: BinaryHeap<(usize, PatientRef)>,
}

impl Hospital {
    #[inline]
    #[must_use]
    pub fn new() -> Self { Self::default() }

    pub fn generate_patient_queue(&mut self) {
        self.patient_queue.clear();

        for patient in self.patients.inorder().iter() {
            let priority = patient.borrow().priority();
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
    pub fn select_doctor(&self, condition: &Condition) -> Option<DoctorRef> {
        let qualified_doctors = self.doctors_by_condition
            .get(&condition.condition)?
            .range(condition.severity..)
            .next()?.1;

        qualified_doctors.iter().find(|doctor| doctor.borrow().available).cloned()
    }

    pub fn assign(doctor: &mut Doctor, patient: &mut Patient, pc: &Condition) {
        println!("Assigning {doctor} to {patient} to treat {pc}");
        doctor.available = false;
        patient.being_treated = true;
    }

    pub fn add_doctor(&mut self, doctor: Doctor) {
        self.doctors.insert(Rc::new(RefCell::new(doctor)));
    }

    pub fn add_patient(&mut self, patient: Patient) {
        self.patients.insert(Rc::new(RefCell::new(patient)));
    }
}

impl Patient {
    /// An aggregate severity score representing the urgency with which this patient needs
    /// treatment.
    pub fn priority(&self) -> usize {
        self.conditions.iter().fold(0, |acc, x| acc + x.severity as usize)
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

#[derive(Debug)]
pub struct Shift {
    /// UNIX time for the start of the shift
    start: usize,
    /// UNIX time for the end of the shift
    end: usize,
}
