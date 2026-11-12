// TODO: SFML

use self::models::*;

mod btree;
mod models;

fn conditions(conditions: impl Into<Vec<(&'static str, u8)>>) -> Vec<Condition> {
    conditions.into().iter()
        .map(|(condition, severity)| Condition::new(*condition, *severity))
        .collect()
}

/// Initialize a [`Hospital`] with hardcoded doctors and patients.
///
/// Data generated from the following sources:
/// * <https://www.name-generator.org.uk/quick/>
/// * <https://www.random.org/calendar-dates/?num=25&start_day=9&start_month=11&start_year=1955&end_day=31&end_month=12&end_year=2003&mondays=on&tuesdays=on&wednesdays=on&thursdays=on&fridays=on&saturdays=on&sundays=on&display=2&format=html&rnd=new>
/// * <https://plagueinc.wiki.gg/wiki/Symptoms>
fn initialize_hospital() -> Hospital {
    let mut hospital = Hospital::new();

    // Doctors
    hospital.add_doctor(Doctor::new(PersonalInformation::new("John",        "Parkinsons",   "1996-12-05"), conditions([("Parkinsons", 10), ("Arthritis", 6)])                           ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Arthur",      "Smith",        "1989-07-15"), conditions([("Leukemia", 8), ("Arthritis", 3)])                              ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Michael",     "Maxwell",      "1988-07-15"), conditions([("Pneumonia", 10)])                                              ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Osman",       "Wills",        "1999-02-25"), conditions([("Cysts", 10), ("Necrosis", 4)])                                 ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Max",         "Watts",        "1980-08-18"), conditions([("Pulmonary Oedema", 10), ("Internal Haemorrhaging", 9)])        ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Vera",        "Martin",       "1986-12-13"), conditions([("Pulmonary Fibrosis", 8)])                                      ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Patrick",     "Coleman",      "1988-02-10"), conditions([("Systemic Infection", 9), ("Necrosis", 10)])                    ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Habiba",      "Matthews",     "1989-03-02"), conditions([("Seizures", 10), ("Epilepsy", 10)])                             ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Tyrese",      "Costello",     "1989-12-31"), conditions([("Total Organ Failure", 8), ("Abscesses", 10), ("Cysts", 7)])    ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Conor",       "Ward",         "1990-03-14"), conditions([("Immune Suppression", 9), ("Coma", 9)])                         ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Chanelle",    "Nicholson",    "1991-10-09"), conditions([("Hypoxia", 10), ("Anemia", 10)])                                ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Tillie",      "Hale",         "1994-05-18"), conditions([("Internal Haemorrhaging", 10)])                                 ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Benedict",    "Skinner",      "1995-07-30"), conditions([("Pulmonary Fibrosis", 8)])                                      ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Thea",        "Mcbride",      "1996-11-11"), conditions([("Hypoxia", 9), ("Total Organ Failure", 10)])                    ));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Hajra",       "Baird",        "1997-11-03"), conditions([("Arthritis", 10), ("Rashes", 10)])                              ));

    // Patients
    hospital.add_patient(Patient::new(PersonalInformation::new("Alex",      "Atwood",       "2003-01-23"), conditions([("Parkinsons", 2)])                                              ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Jason",     "Atwood",       "2003-08-01"), conditions([("Parkinsons", 1), ("Arthritis", 3)])                            ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Jeremy",    "Jeremy",       "2000-01-01"), conditions([("Leukemia", 5)])                                                ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Eli",       "Richards",     "2020-05-05"), conditions([("Pneumonia", 9)])                                               ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Marwa",     "Hubbard",      "1957-03-22"), conditions([("Necrosis", 6), ("Cysts", 1)])                                  ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Kira",      "Alexander",    "1957-08-15"), conditions([("Pulmonary Fibrosis", 5)])                                      ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Callie",    "Burnett",      "1961-12-10"), conditions([("Systemic Infection", 8)])                                      ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Gurniwaz",  "Campbell",     "1962-02-23"), conditions([("Necrosis", 8)])                                                ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Noah",      "Hunter",       "1962-02-25"), conditions([("Cysts", 7)])                                                   ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Che",       "Lamb",         "1962-12-08"), conditions([("Anemia", 6), ("Hypoxia", 4)])                                  ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Kian",      "Chavez",       "1963-10-24"), conditions([("Pulmonary Oedema", 9)])                                        ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Natalia",   "French",       "1966-07-15"), conditions([("Internal Haemorrhaging", 7), ("Systemic Infection", 6)])       ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Deborah",   "Newell",       "1966-10-06"), conditions([("Total Organ Failure", 8)])                                     ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Koby",      "Walsh",        "1966-12-09"), conditions([("Immune Suppression", 4), ("Necrosis", 3)])                     ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Jake",      "Coates",       "1967-06-08"), conditions([("Abscesses", 9)])                                               ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Bartosz",   "Massey",       "1969-03-07"), conditions([("Pneumonia", 5)])                                               ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Zayn",      "Cardenas",     "1970-02-13"), conditions([("Immune Suppression", 9), ("Cysts", 6)])                        ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Wilson",    "Humphrey",     "1973-02-07"), conditions([("Seizures", 7), ("Epilepsy", 9)])                               ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Kobi",      "Le",           "1979-04-19"), conditions([("Coma", 8)])                                                    ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Moshe",     "Goodman",      "1979-07-06"), conditions([("Cysts", 2), ("Rashes", 5)])                                    ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Lacie",     "Cox",          "1979-09-10"), conditions([("Hypoxia", 6)])                                                 ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Gabrielle", "Cuevas",       "1980-07-25"), conditions([("Pulmonary Fibrosis", 8), ("Hypoxia", 9), ("Seizures", 8)])     ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Sunny",     "McKay",        "1988-04-18"), conditions([("Internal Haemorrhaging", 1), ("Parkinsons", 2)])               ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Conner",    "Kline",        "1989-02-23"), conditions([("Rashes", 8)])                                                  ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Adam",      "Knox",         "1991-06-15"), conditions([("Pneumonia", 4)])                                               ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Kaylee",    "Fernandez",    "1993-08-28"), conditions([("Necrosis", 8), ("Cysts", 9), ("Epilepsy", 9), ("Rashes", 9)])  ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Kaiden",    "Cummings",     "1994-04-16"), conditions([("Coma", 4)])                                                    ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Teddy",     "Leonard",      "2001-01-09"), conditions([("Total Organ Failure", 10), ("Coma", 9)])                       ));
    hospital.add_patient(Patient::new(PersonalInformation::new("Esha",      "Gallagher",    "2001-09-05"), conditions([("Parkinsons", 8)])                                              ));

    use std::rc::Rc;
    for doctor in hospital.doctors.inorder().iter() {
        for condition in &doctor.borrow().conditions {
            hospital.doctors_by_condition.entry(Rc::clone(&condition.condition)).or_default()
                .entry(condition.severity)
                .or_default()
                .push(Rc::clone(doctor));
        }
    }

    hospital.generate_patient_queue();
    hospital
}

fn main() {
    let mut hospital = initialize_hospital();

    // TODO: Need some kind of timekeeping. Patients should be released after being treated for a
    // sufficient amount of time, based on the severity of their conditions. Condition severity
    // needs to be reduced after that amount of time for each condition. Need to be able to step
    // through time (even if only forward).
    while let Some((_priority, patient)) = hospital.patient_queue.pop() {
        let mut patient = patient.borrow_mut();

        for pc in patient.conditions.clone() {
            if let Some(doctor) = hospital.select_doctor(&pc) {
                let mut doctor = doctor.borrow_mut();
                Hospital::assign(&mut doctor, &mut patient, &pc);
            }
        }
    }
}
