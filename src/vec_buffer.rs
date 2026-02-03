use bytemuck::{NoUninit, cast_slice};

pub struct VecBuffer<T: NoUninit> {
    label: String,
    usages: wgpu::BufferUsages,

    /// GPU buffer if it has been created.
    pub buffer: wgpu::Buffer,
    /// Current byte size capacity of the GPU buffer.
    capacity: usize,

    phantom: std::marker::PhantomData<T>,
}

impl<T: NoUninit> VecBuffer<T> {
    pub fn with_capacity(
        device: &wgpu::Device,
        capacity: usize,
        label: impl Into<String>,
        usages: wgpu::BufferUsages,
    ) -> Self {
        let label = label.into();

        let buffer =
            Self::create_buffer(device, capacity * std::mem::size_of::<T>(), &label, usages);

        Self {
            label,
            usages,
            buffer,
            capacity,
            phantom: std::marker::PhantomData,
        }
    }

    /// Write the given data to the GPU, resizing the buffer if required. Returns `true` is the buffer was recreated.
    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, items: &[T]) -> bool {
        // Amount of bytes required for all the items.
        let size_in_bytes = std::mem::size_of::<T>() * items.len();
        let mut recreated = false;

        if self.capacity < size_in_bytes {
            let buffer = Self::create_buffer(device, size_in_bytes, &self.label, self.usages);
            self.capacity = size_in_bytes;
            self.buffer = buffer;
            recreated = true;
        }

        queue.write_buffer(&self.buffer, 0, cast_slice(items));

        recreated
    }

    fn create_buffer(
        device: &wgpu::Device,
        size_in_bytes: usize,
        label: &str,
        usages: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: size_in_bytes as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | usages,
            mapped_at_creation: false,
        })
    }
}
