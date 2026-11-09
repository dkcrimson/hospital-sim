# Hospital-Sim

## Overview

This project simulates the operation of a hospital
as practice for working with data structures.
Patients and doctors are added to the hospital at the start,
and a priority queue of patients
(based on total condition severity)
is generated.
Doctors have conditions they are qualified to treat,
up to a certain severity,
and are assigned to patients with those conditions
for an amount of time based on condition severity.

## BTree Implementation

For the BTree,
a `t` value of 6 was chosen
to match the [standard library's choice](https://doc.rust-lang.org/src/alloc/collections/btree/node.rs.html#43).
Their choice is informed by benchmarks.

Although we'd love to have calculated `t` based on page size, the calculation
is trickier since our BTree supports generic data and we don't exactly have
child pointers.

## Visual Component

<!-- TODO: Visual component -->

## Team Member Contributions

<!-- TODO: Team member contributions -->

## UML Class Diagram

<!-- TODO: UML Class Diagram -->
