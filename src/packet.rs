use std::ops::{Deref, DerefMut};

/// Trait for packet buffer allocation and pooling.
///
/// All packet buffers that are allocated from `turbulence` are allocated through this interface.
///
/// Buffers must deref to a `&mut [u8]` and should all have the same length: the MTU for whatever
/// the underlying transport is, up to 32k in size.
pub trait BufferPool {
    type Buffer: Deref<Target = [u8]> + DerefMut;

    fn acquire(&self) -> Self::Buffer;
}

#[derive(Clone, Debug, Default)]
pub struct PacketPool<P>(P);

impl<P> PacketPool<P> {
    pub fn new(buffer_pool: P) -> Self {
        PacketPool(buffer_pool)
    }
}

impl<P: BufferPool> PacketPool<P> {
    pub fn acquire(&self) -> Packet<P::Buffer> {
        Packet {
            buffer: self.0.acquire(),
            len: 0,
        }
    }
}

#[derive(Debug)]
pub struct Packet<B> {
    buffer: B,
    len: usize,
}

impl<B> Packet<B>
where
    B: Deref<Target = [u8]> + DerefMut,
{
    /// Static capacity of this packet
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Resizes the buffer to the given length, panicking if the length is larger than the static
    /// buffer capacity.
    pub fn resize(&mut self, len: usize, val: u8) {
        assert!(len <= self.capacity());
        for i in self.len..len {
            self.buffer[i] = val;
        }
        self.len = len;
    }

    pub fn truncate(&mut self, len: usize) {
        self.len = self.len.min(len);
    }

    pub fn extend(&mut self, other: &[u8]) {
        assert!(self.len + other.len() <= self.capacity());
        self.buffer[self.len..self.len + other.len()].copy_from_slice(other);
        self.len += other.len();
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

impl<B> Deref for Packet<B>
where
    B: Deref<Target = [u8]>,
{
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.buffer[0..self.len]
    }
}

impl<B> DerefMut for Packet<B>
where
    B: Deref<Target = [u8]> + DerefMut,
{
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[0..self.len]
    }
}
