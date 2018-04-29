use melon::{typedef::*, System, VM};
use std::sync::mpsc::{Receiver, Sender};
use ControllerState;

pub struct EnterSystem {
    input_reveicer: Receiver<ControllerState>,
    frame_sender: Sender<Vec<u8>>,
}

impl EnterSystem {
    pub fn new(input: Receiver<ControllerState>, output: Sender<Vec<u8>>) -> EnterSystem {
        EnterSystem {
            input_reveicer: input,
            frame_sender: output,
        }
    }
}

const NUM_FRAME_BYTES: usize = 32 * 32;
const SIGNAL_FLUSH_FRAME: u16 = 1;

impl System for EnterSystem {
    const ID: &'static str = "ENTER32";

    const MEM_PAGES: u8 = 2;

    fn prepare(&mut self, vm: &mut VM) -> Result<()> {
        vm.alloc(Self::MEM_PAGES as u16 * 1024)
    }

    fn pre_cycle(&mut self, vm: &mut VM) -> Result<()> {
        if let Ok(ctrl) = self.input_reveicer.try_recv() {
            vm.mem[NUM_FRAME_BYTES] = ctrl.a as u8;
            vm.mem[NUM_FRAME_BYTES + 1] = ctrl.b as u8;
            vm.mem[NUM_FRAME_BYTES + 2] = ctrl.x as u8;
            vm.mem[NUM_FRAME_BYTES + 3] = ctrl.y as u8;
            vm.mem[NUM_FRAME_BYTES + 4] = ctrl.start as u8;
            vm.mem[NUM_FRAME_BYTES + 5] = ctrl.select as u8;
            vm.mem[NUM_FRAME_BYTES + 6] = ctrl.right_trigger as u8;
            vm.mem[NUM_FRAME_BYTES + 7] = ctrl.left_trigger as u8;
            vm.mem[NUM_FRAME_BYTES + 8] = ctrl.d_pad_up as u8;
            vm.mem[NUM_FRAME_BYTES + 9] = ctrl.d_pad_down as u8;
            vm.mem[NUM_FRAME_BYTES + 10] = ctrl.d_pad_left as u8;
            vm.mem[NUM_FRAME_BYTES + 11] = ctrl.d_pad_right as u8;
        }

        Ok(())
    }

    fn system_call(&mut self, vm: &mut VM, signal: u16) -> Result<()> {
        match signal {
            SIGNAL_FLUSH_FRAME => {
                let current_frame = &vm.mem[0..NUM_FRAME_BYTES];

                self.frame_sender.send(current_frame.to_vec())?;
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}
