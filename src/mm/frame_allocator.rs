use core::cell::RefCell;

use alloc::{rc::Rc, vec::Vec};

use crate::mm::{MAX_FRAME, PAGE_SIZE};

use super::Frame;

pub type SharedFrameAllocator = Rc<RefCell<FrameAllocator>>;

/// physical number

/// representing a physical memory address

pub struct FrameAllocator {
    current: usize,
    end: usize,
    recycle: Vec<Frame>,
}

#[derive(Debug)]
pub enum FrameError {
    NoAligned,

    Invalid,
}

pub type FrameResult<T> = Result<T, FrameError>;

impl FrameAllocator {
    pub fn new(start: usize, end: usize) -> FrameResult<SharedFrameAllocator> {
        extern "C" {
            /// the position end of the kernel
            fn ekernel();
        }

        if start > end || start < ekernel as usize || end > MAX_FRAME {
            return Err(FrameError::Invalid);
        }

        if end - start < PAGE_SIZE {
            return Err(FrameError::NoAligned);
        }

        let mut head = start;
        let mut rest = head % PAGE_SIZE;
        if rest != 0 {
            head += PAGE_SIZE - rest;
        }

        let mut tail = end;
        rest = tail % PAGE_SIZE;
        if rest != 0 {
            tail -= rest;
        }

        Ok(Rc::new(RefCell::new(Self {
            current: head,
            end: tail,
            recycle: Vec::new(),
        })))
    }

    pub(super) fn alloc(&mut self) -> Option<Frame> {
        if !self.recycle.is_empty() {
            self.recycle.pop()
        } else if self.current >= self.end {
            None
        } else {
            self.current += PAGE_SIZE;
            Some((self.current - PAGE_SIZE).try_into().unwrap())
        }
    }

    pub(super) fn dealloc(&mut self, frame: Frame) {
        self.recycle.push(frame);
    }
}
