pub(crate) mod terminal;

use std::io::{self, Write};

use crate::snapshot::Snapshot;

pub(crate) fn json(out: &mut impl Write, snapshot: &Snapshot<'_>) -> io::Result<()> {
    serde_json::to_writer(&mut *out, snapshot).map_err(io::Error::from)?;
    writeln!(out)
}
