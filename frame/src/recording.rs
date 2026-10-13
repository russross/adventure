//! Compact observed input changes, timed by logical reads rather than host events.

use std::collections::{BTreeMap, VecDeque};
use std::io::{BufRead, Write};

use adventure_trace::Control;

#[derive(Clone, Copy, Debug)]
struct Change {
    read: u64,
    control: Control,
    value: u8,
}

// Defaults need no records; controls retain their last observed value during playback.
fn default_value(control: Control) -> u8 {
    u8::from(control == Control::Color)
}

pub struct Recording {
    reads: u64,
    frames: u64,
    values: BTreeMap<Control, u8>,
    changes: Option<VecDeque<Change>>,
    writer: Option<Box<dyn Write>>,
    error: Option<String>,
    canceled_boot: bool,
}

impl Recording {
    pub fn record_to(mut writer: impl Write + 'static) -> Result<Self, String> {
        writeln!(writer, "adventure-input 1").map_err(|e| e.to_string())?;
        Ok(Self {
            reads: 0,
            frames: 0,
            values: BTreeMap::new(),
            changes: None,
            writer: Some(Box::new(writer)),
            error: None,
            canceled_boot: false,
        })
    }

    // Parse the complete file before opening SDL. Only input values and the endpoint are stored.
    pub fn replay(reader: impl BufRead) -> Result<Self, String> {
        let mut lines = reader.lines();
        if lines
            .next()
            .transpose()
            .map_err(|e| e.to_string())?
            .as_deref()
            != Some("adventure-input 1")
        {
            return Err("unsupported input recording header".into());
        }
        let mut changes = VecDeque::new();
        let mut end = None;
        for (index, line) in lines.enumerate() {
            let line = line.map_err(|e| e.to_string())?;
            let tokens: Vec<_> = line.split_whitespace().collect();
            if end.is_some() {
                return Err("data after recording end".into());
            }
            let invalid = || format!("invalid input recording at line {}", index + 2);
            match tokens.as_slice() {
                ["input", read, control, value] => {
                    let change = Change {
                        read: read.parse().map_err(|_| invalid())?,
                        control: control.parse()?,
                        value: value.parse().map_err(|_| invalid())?,
                    };
                    change.control.validate(change.value)?;
                    if changes
                        .back()
                        .is_some_and(|last: &Change| last.read >= change.read)
                    {
                        return Err("input read indices must increase".into());
                    }
                    changes.push_back(change);
                }
                ["end", frames, reads] => {
                    end = Some((
                        frames.parse::<u64>().map_err(|_| invalid())?,
                        reads.parse::<u64>().map_err(|_| invalid())?,
                    ));
                }
                _ => return Err(invalid()),
            }
        }
        let (frames, reads) = end.ok_or("input recording needs an explicit end")?;
        if changes.back().is_some_and(|change| change.read >= reads) {
            return Err("input change outside recording".into());
        }
        Ok(Self {
            reads: 0,
            frames,
            values: BTreeMap::new(),
            changes: Some(changes),
            writer: None,
            error: None,
            canceled_boot: reads == 0,
        })
    }

    pub fn frames(&self) -> u64 {
        self.frames
    }

    pub fn stopped_at_boot(&self) -> bool {
        self.canceled_boot
    }

    // Every logical read advances the clock, even when its value needs no storage.
    pub fn input(&mut self, control: Control, live: u8) -> u8 {
        let previous = self
            .values
            .get(&control)
            .copied()
            .unwrap_or_else(|| default_value(control));
        let value = if let Some(changes) = &mut self.changes {
            while changes
                .front()
                .is_some_and(|change| change.read <= self.reads)
            {
                let change = changes.pop_front().expect("front exists");
                self.values.insert(change.control, change.value);
            }
            self.values.get(&control).copied().unwrap_or(previous)
        } else {
            if live != previous {
                self.write_line(format!("input {} {control} {live}", self.reads));
            }
            self.values.insert(control, live);
            live
        };
        self.reads += 1;
        value
    }

    pub fn next_frame(&mut self) {
        if self.changes.is_none() {
            self.frames += 1;
        }
    }

    fn write_line(&mut self, line: String) {
        if self.error.is_none()
            && let Some(writer) = &mut self.writer
            && let Err(error) = writeln!(writer, "{line}")
        {
            self.error = Some(format!("cannot write input recording: {error}"));
        }
    }

    pub fn check(&self) -> Result<(), String> {
        self.error.clone().map_or(Ok(()), Err)
    }

    // Playback does not compare state, read counts, or generated output with the recording.
    pub fn finish(&mut self) -> Result<(), String> {
        if self.writer.is_some() {
            self.write_line(format!("end {} {}", self.frames, self.reads));
        }
        self.check()?;
        if let Some(writer) = &mut self.writer {
            writer
                .flush()
                .map_err(|e| format!("cannot flush input recording: {e}"))?;
        }
        Ok(())
    }
}
