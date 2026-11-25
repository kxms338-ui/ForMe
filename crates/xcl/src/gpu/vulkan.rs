
// Backend Vulkan indépendant pour XCL.

use ash::{vk, Entry};
use std::ffi::CStr;

pub struct VulkanContext {
    pub entry: Entry,
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub graphics_queue: vk::Queue,
    pub compute_queue: vk::Queue,
    pub graphics_queue_family: u32,
    pub compute_queue_family: u32,
}

impl VulkanContext {
    pub fn new() -> Result<Self, String> {
        let entry = unsafe {
            Entry::load().map_err(|e| e.to_string())?
        };

        let app_info = vk::ApplicationInfo::default()
            .api_version(vk::API_VERSION_1_2);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info);

        let instance = unsafe {
            entry.create_instance(&create_info, None)
                .map_err(|e| format!("Instance Vulkan : {e:?}"))?
        };

        let devices = unsafe {
            instance.enumerate_physical_devices()
                .map_err(|e| format!("Détection GPU : {e:?}"))?
        };

        let mut selected = None;

        for physical_device in devices {
            let queues = unsafe {
                instance.get_physical_device_queue_family_properties(
                    physical_device
                )
            };

            let graphics_family = queues.iter().position(|q| {
                q.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            });

            let dedicated_compute = queues.iter().position(|q| {
                q.queue_flags.contains(vk::QueueFlags::COMPUTE)
                    && !q.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            });

            let any_compute = queues.iter().position(|q| {
                q.queue_flags.contains(vk::QueueFlags::COMPUTE)
            });

            if let (Some(graphics), Some(compute)) =
                (graphics_family, dedicated_compute.or(any_compute))
            {
                selected = Some((
                    physical_device,
                    graphics as u32,
                    compute as u32,
                ));
                break;
            }
        }

        let (physical_device, graphics_queue_family, compute_queue_family) =
            selected.ok_or("Aucun GPU compatible avec les files graphiques et calcul")?;

        let priorities = [1.0_f32];

        let mut queue_infos = vec![
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(graphics_queue_family)
                .queue_priorities(&priorities),
        ];

        if compute_queue_family != graphics_queue_family {
            queue_infos.push(
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(compute_queue_family)
                    .queue_priorities(&priorities),
            );
        }

        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos);

        let device = unsafe {
            instance.create_device(
                physical_device,
                &device_info,
                None,
            ).map_err(|e| format!("Création périphérique Vulkan : {e:?}"))?
        };

        let graphics_queue = unsafe {
            device.get_device_queue(graphics_queue_family, 0)
        };

        let compute_queue = unsafe {
            device.get_device_queue(compute_queue_family, 0)
        };

        Ok(Self {
            entry,
            instance,
            physical_device,
            device,
            graphics_queue,
            compute_queue,
            graphics_queue_family,
            compute_queue_family,
        })
    }

    pub fn device_name(&self) -> String {
        let properties = unsafe {
            self.instance.get_physical_device_properties(
                self.physical_device
            )
        };

        unsafe {
            CStr::from_ptr(properties.device_name.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_vulkan_gpu() {
        let context = VulkanContext::new().unwrap();
        let name = context.device_name();

        println!("GPU Vulkan détecté : {name}");
        assert!(!name.is_empty());
    }

    #[test]
    fn inspect_vulkan_capabilities() {
        let context = VulkanContext::new().unwrap();

        println!(
            "File graphique : {}",
            context.graphics_queue_family
        );

        println!(
            "File calcul : {}",
            context.compute_queue_family
        );

        let memory = unsafe {
            context.instance
                .get_physical_device_memory_properties(
                    context.physical_device
                )
        };

        for i in 0..memory.memory_heap_count {
            let heap = memory.memory_heaps[i as usize];

            println!(
                "Mémoire GPU : {} Mo",
                heap.size / 1024 / 1024
            );
        }
    }

    #[test]
    fn create_logical_device() {
        let context = VulkanContext::new().unwrap();

        println!("GPU : {}", context.device_name());
        println!("Périphérique logique Vulkan créé !");
        println!(
            "File graphique : {}",
            context.graphics_queue_family
        );
        println!(
            "File calcul : {}",
            context.compute_queue_family
        );

        assert_ne!(context.graphics_queue, vk::Queue::null());
        assert_ne!(context.compute_queue, vk::Queue::null());
    }
}
