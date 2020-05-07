// pub mod memory {
use std::mem;
#[derive(Debug)]
pub struct Memory {
    pool: Vec<usize>,
    heap: Vec<Vec<u32>>,
}

const PROGRAM_ADDRESS: usize = 0;
impl Memory {
    // create a new Memory, comprising a pool of reusable IDs
    // and a heap of UM words, populated with the instructions
    // as segment 0
    pub fn new(instructions: Vec<u32>) -> Memory {
        Memory { pool: Vec::new(), heap: vec![instructions] }
    }

    // allocate and initalize (as all 0s) a memory segment.
    // returns the segment ID
    pub fn allocate(&mut self, size: usize) -> usize {
        let space = vec![0_u32; size];
        // can we reuse a previously unmapped segment id?
        if self.pool.len() == 0 {
            self.heap.push(space);
            self.heap.len() - 1
        } else {
            let address = self.pool.pop().expect("No segment ID available");

            mem::replace(
                self.heap
                    .get_mut(address)
                    .expect("Memory was not previously allocated"),
                space,
            );

            address
        }
    }

    // deallocate the memory at the given address.
    pub fn deallocate(&mut self, address: usize) {
        self.pool.push(address);
        mem::replace(
            self.heap
                .get_mut(address)
                .expect("Memory was not previously allocated"),
            Vec::new(),
        );
    }

    // supply contents of the memory at the given address if
    // initialized, None otherwise.
    #[inline]
    pub fn load(&self, seg_id: usize, address: usize) -> Option<u32> {
        match self.heap.get(seg_id) {
            Some(segment) => Some(segment[address]),
            None => panic!("Segment unmapped!"),
        }
    }
    // get the instruction word corresponding to the given program counter
    // This may have high overhead...
    #[inline]
    pub fn get_instruction(&self, pc: usize) -> u32 {
        match self.heap.get(PROGRAM_ADDRESS) {
            Some(program) => program[pc],
            None => panic!("Program was unallocated"),
        }
    }

    // write a value into the given address of the given segment.
    #[inline]
    pub fn store(&mut self, seg_id: usize, address: usize, value: u32) {
        let memory =
            self.heap.get_mut(seg_id).expect("Memory was unallocated");

        mem::replace(
            memory
                .get_mut(address)
                .expect("No value present at given address"),
            value,
        );
    }

    // replace the program with the vector at the given address
    pub fn load_segment(&mut self, seg_id: usize) {
        let program = self
            .heap
            .get(seg_id)
            .expect("Found no program at the given address")
            .clone();

        mem::replace(
            self.heap
                .get_mut(PROGRAM_ADDRESS)
                .expect("Found no existing program"),
            program,
        );
    }
}
// }
