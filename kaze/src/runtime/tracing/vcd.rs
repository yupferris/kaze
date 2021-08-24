//! [VCD](https://en.wikipedia.org/wiki/Value_change_dump) format tracing implementation.

extern crate vcd;

use super::*;

use std::io;

pub enum TimeScaleUnit {
    S,
    Ms,
    Us,
    Ns,
    Ps,
    Fs,
}

impl From<TimeScaleUnit> for vcd::TimescaleUnit {
    fn from(time_scale_unit: TimeScaleUnit) -> Self {
        match time_scale_unit {
            TimeScaleUnit::S => vcd::TimescaleUnit::S,
            TimeScaleUnit::Ms => vcd::TimescaleUnit::MS,
            TimeScaleUnit::Us => vcd::TimescaleUnit::US,
            TimeScaleUnit::Ns => vcd::TimescaleUnit::NS,
            TimeScaleUnit::Ps => vcd::TimescaleUnit::PS,
            TimeScaleUnit::Fs => vcd::TimescaleUnit::FS,
        }
    }
}

pub struct VcdTrace<W: io::Write> {
    module_hierarchy_depth: u32,

    signals: Vec<VcdTraceSignal>,

    w: vcd::Writer<W>,
}

impl<W: io::Write> VcdTrace<W> {
    pub fn new(w: W, time_scale: u32, time_scale_unit: TimeScaleUnit) -> io::Result<VcdTrace<W>> {
        let mut w = vcd::Writer::new(w);

        w.timescale(time_scale, time_scale_unit.into())?;

        Ok(VcdTrace {
            module_hierarchy_depth: 0,

            signals: Vec::new(),

            w,
        })
    }
}

impl<W: io::Write> Trace for VcdTrace<W> {
    type SignalId = usize;

    fn push_module(&mut self, name: &'static str) -> io::Result<()> {
        self.w.add_module(name)?;

        self.module_hierarchy_depth += 1;

        Ok(())
    }

    fn pop_module(&mut self) -> io::Result<()> {
        self.w.upscope()?;

        self.module_hierarchy_depth -= 1;

        if self.module_hierarchy_depth == 0 {
            self.w.enddefinitions()?;
        }

        Ok(())
    }

    fn add_signal(
        &mut self,
        name: &'static str,
        bit_width: u32,
        type_: TraceValueType,
    ) -> io::Result<Self::SignalId> {
        let ret = self.signals.len();

        self.signals.push(VcdTraceSignal {
            bit_width,
            type_,
            // TODO: Is wire the right construct here always?
            id: self.w.add_wire(bit_width, name)?,
        });

        Ok(ret)
    }

    fn update_time_stamp(&mut self, time_stamp: u64) -> io::Result<()> {
        self.w.timestamp(time_stamp)
    }

    fn update_signal(&mut self, signal_id: &Self::SignalId, value: TraceValue) -> io::Result<()> {
        // TODO: Type check incoming value!
        let signal = &self.signals[*signal_id];

        if let TraceValueType::Bool = signal.type_ {
            self.w.change_scalar(
                signal.id,
                match value {
                    TraceValue::Bool(value) => value,
                    TraceValue::U32(_) | TraceValue::U64(_) | TraceValue::U128(_) => unreachable!(),
                },
            )?;
        } else {
            let value = match value {
                TraceValue::Bool(_) => unreachable!(),
                TraceValue::U32(value) => value as _,
                TraceValue::U64(value) => value as _,
                TraceValue::U128(value) => value,
            };
            let mut scalar_values = [vcd::Value::V0; 128];
            for (i, scalar_value) in scalar_values.iter_mut().enumerate() {
                *scalar_value = ((value >> (signal.bit_width as usize - 1 - i)) & 1 != 0).into();
            }
            self.w
                .change_vector(signal.id, &scalar_values[0..signal.bit_width as usize])?;
        }

        Ok(())
    }
}

struct VcdTraceSignal {
    bit_width: u32,
    type_: TraceValueType,
    id: vcd::IdCode,
}
