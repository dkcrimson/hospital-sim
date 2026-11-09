use self::models::*;

mod btree;
mod models;

fn main() {
    let mut hospital = Hospital::new();

    // Doctors
    hospital.add_doctor(Doctor::new(PersonalInformation::new("John",     "Parkinsons",  "1996-12-05"), [Condition::new("Parkinsons", 10), Condition::new("Arthritis", 6)]));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Arthur",   "Smith",       "1989-07-15"), [Condition::new("Leukemia", 8), Condition::new("Arthritis", 3)]));
    hospital.add_doctor(Doctor::new(PersonalInformation::new("Michael",  "Maxwell",     "1988-07-15"), [Condition::new("Pneumonia", 10)]));

    // Patients
    hospital.add_patient(Patient::new(PersonalInformation::new("Alex",   "Atwood",      "2003-01-23"), [Condition::new("Parkinsons", 2)]));
    hospital.add_patient(Patient::new(PersonalInformation::new("Jason",  "Atwood",      "2003-08-01"), [Condition::new("Parkinsons", 1), Condition::new("Arthritis", 3)]));
    hospital.add_patient(Patient::new(PersonalInformation::new("Jeremy", "Jeremy",      "2000-01-01"), [Condition::new("Leukemia", 5)]));
    hospital.add_patient(Patient::new(PersonalInformation::new("Eli",    "Richards",    "2020-05-05"), [Condition::new("Pneumonia", 9)]));

    hospital.generate_patient_queue();

    while let Some((_priority, patient)) = hospital.patient_queue.pop() {
        let mut patient = patient.borrow_mut();

        for pc in patient.conditions.clone() {
            if let Some(doctor) = hospital.doctors.inorder().iter().find(|doctor| doctor.borrow().available && doctor.borrow().conditions.iter().find(|dc| dc.condition == pc.condition && dc.severity >= pc.severity).is_some()) {
                let mut doctor = doctor.borrow_mut();
                Hospital::assign(&mut doctor, &mut patient, &pc);
            }
        }
    }
}
