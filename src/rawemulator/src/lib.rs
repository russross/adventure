//! Original-cartridge validation with resumable frame boundaries.

mod bus;
mod cpu;
mod tia;

use adventure_trace::Record;
use bus::Bus;
use cpu::Cpu;

pub struct Validator {
    cpu: Cpu,
    paused: bool,
}

impl Validator {
    pub fn new(records: Vec<Record>) -> Result<Self, String> {
        Ok(Self {
            cpu: Cpu::new(Bus::stream(records)?),
            paused: false,
        })
    }

    // Resume the previous boundary, then stop before the next frame begins.
    pub fn catch_up(&mut self, records: Vec<Record>, frame: u64) -> Result<(), String> {
        self.cpu.bus.session.append(records);
        if self.paused {
            self.step()?;
            self.paused = false;
        }
        for _ in 0..100_000 {
            if self.cpu.bus.at_boundary(self.cpu.pc) && self.cpu.bus.session.frame == frame {
                self.cpu.bus.session.check()?;
                if self.cpu.bus.session.pending() {
                    return Err(format!("ROM missing observations before frame {frame}"));
                }
                self.paused = true;
                return Ok(());
            }
            self.step()?;
        }
        Err(format!(
            "ROM instruction budget exceeded at {:04x}",
            self.cpu.pc
        ))
    }

    // The final attempted presentation checks all work since the last accepted picture.
    pub fn finish(&mut self, records: Vec<Record>, frame: u64) -> Result<(), String> {
        self.cpu.bus.session.append(records);
        self.cpu.bus.stop_at(frame);
        for _ in 0..100_000 {
            if !self.step()? {
                return Ok(());
            }
        }
        Err(format!(
            "ROM instruction budget exceeded at {:04x}",
            self.cpu.pc
        ))
    }

    fn step(&mut self) -> Result<bool, String> {
        self.cpu.step().map_err(|error| {
            format!(
                "{error} (ROM PC={:04x}, {})",
                self.cpu.pc,
                self.cpu.bus.diagnostics()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Validator;

    // A plausible main-game event cannot replace independently executed ROM state.
    #[test]
    fn incorrect_live_initialization_fails_at_original_instruction() {
        let boot = "00:00:00/boot#0 trace 1 adventure-original boot"
            .parse()
            .unwrap();
        let wrong_game = "00:00:00/init#0 init 2".parse().unwrap();
        let mut validator = Validator::new(vec![boot]).unwrap();
        let error = validator.catch_up(vec![wrong_game], 0).unwrap_err();
        assert!(error.contains("expected 00:00:00/init#0 init 2"), "{error}");
        assert!(error.contains("observed 00:00:00/init#0 init 1"), "{error}");
        assert!(error.contains("ROM PC=f3d5"), "{error}");
    }
}
