mod profile_rw;
mod report;

use crate::device::checksum;
use report::{make_request, StandardReport};

// Checksum algorithms used.
type Sum171 = checksum::SumComplement8<171>;
type Sum181 = checksum::SumComplement8<181>;
