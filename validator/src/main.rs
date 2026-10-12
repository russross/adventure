//! Headless validation of a semantic trace against the original cartridge instruction stream.

mod bus;
mod cpu;
mod tia;

#[cfg(test)]
#[path = "../../tests/trace_fixture.rs"]
pub mod fixture;

use adventure_trace::Record;
use bus::Bus;
use cpu::Cpu;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

fn execute() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let path = args.next().ok_or("usage: adventure-validator TRACE")?;
    if args.next().is_some() {
        return Err("usage: adventure-validator TRACE".into());
    }
    let records = BufReader::new(File::open(&path).map_err(|e| format!("{path}: {e}"))?)
        .lines()
        .enumerate()
        .map(|(index, line)| {
            line.map_err(|e| e.to_string())?
                .parse::<Record>()
                .map_err(|e| format!("{path}:{}: {e}", index + 1))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let frames = records.last().ok_or("empty trace")?.at.frame;
    let mut cpu = Cpu::new(Bus::new(records)?);
    let budget = frames
        .checked_add(1)
        .and_then(|v| v.checked_mul(100_000))
        .ok_or("trace duration overflow")?;
    for _ in 0..budget {
        match cpu.step() {
            Ok(true) => {}
            Ok(false) => {
                println!("ROM verified {frames} frames");
                return Ok(());
            }
            Err(e) => return Err(format!("{e} (PC={:04x})", cpu.pc)),
        }
    }
    Err(format!("ROM instruction budget exceeded at {:04x}", cpu.pc))
}
fn main() {
    if let Err(error) = execute() {
        eprintln!("Adventure validator: {error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{Bus, Cpu, fixture};

    #[test]
    fn canceled_boot_trace_validates_without_initialization() {
        let mut recording = adventure_trace::Session::record();
        recording.finish().unwrap();
        let mut cpu = Cpu::new(Bus::new(recording.records().to_vec()).unwrap());
        assert!(!cpu.step().unwrap());
    }

    #[test]
    fn original_instruction_stream_validates_translation() {
        for game in 1..=3 {
            for frames in [1, 3, 33, 105, 401] {
                let (_, records) = fixture::record(game, frames, 2);
                let mut cpu = Cpu::new(Bus::new(records).unwrap());
                let mut complete = false;
                for _ in 0..(frames + 1) * 100_000 {
                    match cpu.step() {
                        Ok(true) => {}
                        Ok(false) => {
                            complete = true;
                            break;
                        }
                        Err(e) => panic!("game={game} frames={frames} PC={:04x}: {e}", cpu.pc),
                    }
                }
                assert!(complete, "ROM failed to reach trace end");
            }
        }
    }

    #[test]
    fn original_gate_contacts_validate_translation() {
        let (game, records) = fixture::record_castle();
        assert_eq!(
            game.ram.player.room,
            adventure_port::adventure::Room::GoldTreasury,
            "player={:?}, gate={:?}, held={:?}",
            game.ram.player,
            game.ram.gates.gold,
            game.ram.held
        );
        let mut cpu = Cpu::new(Bus::new(records).unwrap());
        for _ in 0..20_000_000 {
            match cpu.step() {
                Ok(true) => {}
                Ok(false) => return,
                Err(e) => panic!("PC={:04x}: {e}", cpu.pc),
            }
        }
        panic!("ROM failed to reach trace end");
    }
}
