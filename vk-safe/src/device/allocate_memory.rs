use super::*;
use vk_safe_sys as vk;

use crate::physical_device::MemoryTypeChoice;
use crate::scope::{ScopeId, ScopeLife};

use vk::has_command::{AllocateMemory, FreeMemory};

use std::mem::MaybeUninit;
use std::ops::Deref;

pub trait DeviceMemoryConfig: Deref<Target = Self::Device> {
    type FreeProvider;
    type Commands: FreeMemory<Self::FreeProvider>;
    type Device: Device<Commands = Self::Commands>;
}

pub struct Config<D, F> {
    device: D,
    free_provider: PhantomData<F>,
}

impl<D: Device, F> DeviceMemoryConfig for Config<D, F>
where
    D::Commands: FreeMemory<F>,
{
    type FreeProvider = F;
    type Commands = D::Commands;
    type Device = D;
}

impl<D: Device, F> Deref for Config<D, F> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

pub trait DeviceMemory: Deref<Target = DeviceMemoryType<Self::Config>> {
    type Config: DeviceMemoryConfig<Device = Self::Device>;
    type Device;
}

pub struct DeviceMemoryType<D: DeviceMemoryConfig> {
    pub(crate) handle: vk::DeviceMemory,
    device: D,
}

impl<D: DeviceMemoryConfig> DeviceMemoryType<D> {
    fn new(handle: vk::DeviceMemory, device: D) -> Self {
        Self { handle, device }
    }
}

impl<D: DeviceMemoryConfig> std::fmt::Debug for DeviceMemoryType<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl<'d, 'pd, C: DeviceConfig, Pd: PhysicalDevice + ScopeLife<'pd>> ScopedDeviceType<'d, C, Pd> {
    pub fn allocate_memory<P, F>(
        &self,
        info: &MemoryAllocateInfo<'pd>,
    ) -> Result<DeviceMemoryType<Config<Self, F>>, vk::Result>
    where
        C::Commands: AllocateMemory<P> + FreeMemory<F>,
    {
        let fptr = self.commands.AllocateMemory().get_fptr();
        let mut memory = MaybeUninit::uninit();
        unsafe {
            let ret = fptr(
                self.handle,
                &info.inner,
                std::ptr::null(),
                memory.as_mut_ptr(),
            );
            check_raw_err!(ret);
            Ok(DeviceMemoryType::new(
                memory.assume_init(),
                Config {
                    device: *self,
                    free_provider: PhantomData,
                },
            ))
        }
    }
}

impl<D: DeviceMemoryConfig> Drop for DeviceMemoryType<D> {
    fn drop(&mut self) {
        let fptr = self.device.commands.FreeMemory().get_fptr();
        unsafe {
            fptr(self.device.handle, self.handle, std::ptr::null());
        }
    }
}

pub struct MemoryAllocateInfo<'pd> {
    inner: vk::MemoryAllocateInfo,
    _pd: ScopeId<'pd>,
}

impl<'pd> MemoryAllocateInfo<'pd> {
    pub const fn new(
        size: std::num::NonZeroU64,
        memory_type_choice: MemoryTypeChoice<'pd>,
    ) -> Self {
        #![allow(unused_labels)]
        check_vuids::check_vuids!(MemoryAllocateInfo);

        'VUID_VkMemoryAllocateInfo_allocationSize_00638: {
            check_vuids::version!("1.2.3");
            check_vuids::description!("hey there");

            let _ = 0; // test code
        }

        let inner = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            allocation_size: size.get(),
            memory_type_index: memory_type_choice.index,
        };

        Self {
            inner,
            _pd: ScopeId::new(),
        }
    }
}
