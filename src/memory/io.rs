use crate::memory::{Storage, StorageMut};

pub trait IOHandler {
    fn can_write(&self, memory: &[u8], address: u32, size: u32) -> bool;

    fn on_write(&self, memory: &[u8], address: u32, size: u32);
}

pub struct IOMemory<H: IOHandler> {
    memory: Vec<u8>,
    handler: H,
}

impl<H: IOHandler> IOMemory<H> {
    pub fn new(size: u32, handler: H) -> IOMemory<H> {
        IOMemory {
            memory: vec![0; size as usize],
            handler,
        }
    }
}

impl<H: IOHandler> Storage for IOMemory<H> {
    fn length(&self) -> u32 {
        self.memory.length()
    }

    fn check_range(&self, address: u32, length: u32) -> bool {
        self.memory.check_range(address, length)
    }

    fn borrow_slice(&self, address: u32, length: u32) -> Result<&[u8], ()> {
        self.memory.borrow_slice(address, length)
    }
}

impl<H: IOHandler> StorageMut for IOMemory<H> {
    fn write(&mut self, address: u32, size: u32, value: u32) -> Result<(), ()> {
        if self.handler.can_write(&self.memory, address, size) {
            self.memory.write(address, size, value)?;
            self.handler.on_write(&self.memory, address, size);
        }
        Ok(())
    }
}

pub struct DelegateIOHandler<FC, FO>
where
    FC: Fn(&[u8], u32, u32) -> bool,
    FO: Fn(&[u8], u32, u32),
{
    can_write: FC,
    on_write: FO,
}

impl<FC, FO> DelegateIOHandler<FC, FO>
where
    FC: Fn(&[u8], u32, u32) -> bool,
    FO: Fn(&[u8], u32, u32),
{
    pub fn new(can_write: FC, on_write: FO) -> DelegateIOHandler<FC, FO> {
        DelegateIOHandler {
            can_write,
            on_write,
        }
    }
}

impl<FC, FO> IOHandler for DelegateIOHandler<FC, FO>
where
    FC: Fn(&[u8], u32, u32) -> bool,
    FO: Fn(&[u8], u32, u32),
{
    fn can_write(&self, memory: &[u8], address: u32, size: u32) -> bool {
        (self.can_write)(memory, address, size)
    }

    fn on_write(&self, memory: &[u8], address: u32, size: u32) {
        (self.on_write)(memory, address, size)
    }
}

#[cfg(test)]
mod tests {
    use super::{DelegateIOHandler, IOMemory};
    use crate::memory::Storage;
    use crate::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn write_callback() {
        let result = Rc::new(Cell::new((0u32, 0u32)));
        let rref = Rc::clone(&result);

        let handler = DelegateIOHandler::new(
            |_, _, _| true,
            move |memory, address, size| {
                let value = memory.read(address, size).unwrap();
                rref.set((address, value));
            },
        );

        let program = program_from_words(&[
            instr_i(OpCode::LI, RegisterId::T0, RegisterId::ZERO, 923),
            instr_i(OpCode::SW, RegisterId::T0, RegisterId::ZERO, 4),
            instr_i(OpCode::HALT, RegisterId::ZERO, RegisterId::ZERO, 0),
        ]);

        let mut processor = Processor::default();
        let mut memory = IOMemory::new(16, handler);

        assert_eq!(processor.run(&program, &mut memory), ExitCode::Halted);

        let (address, value) = result.get();

        assert_eq!(address, 4u32);
        assert_eq!(value, 923u32);
    }
}
