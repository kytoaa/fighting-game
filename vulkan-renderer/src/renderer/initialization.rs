use ash::{vk, Device, Entry, Instance};

use super::*;

impl CoreRenderData {
    pub fn init(
        display_handle: &dyn winit::raw_window_handle::HasDisplayHandle,
        window_handle: &dyn winit::raw_window_handle::HasWindowHandle,
        window_size: (u32, u32),
    ) -> Self {
        unsafe {
            let entry = Entry::linked();
            let app_name = c"FightingGame";

            let layer_names = [c"VK_LAYER_KHRONOS_validation"];
            let layers_names_raw: Vec<*const std::ffi::c_char> = layer_names
                .iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();

            let mut extension_names = ash_window::enumerate_required_extensions(
                display_handle
                    .display_handle()
                    .expect("failed to get display handle")
                    .as_raw(),
            )
            .unwrap()
            .to_vec();
            extension_names.push(ash::ext::debug_utils::NAME.as_ptr());

            let instance: Instance = {
                let appinfo = vk::ApplicationInfo::default()
                    .application_name(app_name)
                    .application_version(0)
                    .engine_name(app_name)
                    .engine_version(0)
                    .api_version(vk::make_api_version(0, 1, 0, 0));

                let create_flags = vk::InstanceCreateFlags::default();

                let create_info = vk::InstanceCreateInfo::default()
                    .application_info(&appinfo)
                    .enabled_layer_names(&layers_names_raw)
                    .enabled_extension_names(&extension_names)
                    .flags(create_flags);

                entry
                    .create_instance(&create_info, None)
                    .expect("Instance creation error")
            };

            let debug_utils_instance = ash::ext::debug_utils::Instance::new(&entry, &instance);
            // NOTE: debug info setup
            let debug_callback = {
                let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                    .message_severity(
                        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                            | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                    )
                    .message_type(
                        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                    )
                    .pfn_user_callback(Some(vulkan_debug_callback));

                debug_utils_instance
                    .create_debug_utils_messenger(&debug_info, None)
                    .unwrap()
            };

            let surface = ash_window::create_surface(
                &entry,
                &instance,
                display_handle
                    .display_handle()
                    .expect("failed to get display handle")
                    .as_raw(),
                window_handle
                    .window_handle()
                    .expect("failed to get window handle")
                    .as_raw(),
                None,
            )
            .unwrap();

            let pdevices = instance
                .enumerate_physical_devices()
                .expect("Physical device error");
            let surface_instance = ash::khr::surface::Instance::new(&entry, &instance);

            struct QueueFamilyInfo {
                graphics: Option<usize>,
                transfer: Option<usize>,
            }
            let (physical_device, queue_family_index, transfer_queue_family_index) =
                find_physical_device(&instance, &surface_instance, &surface, &pdevices)
                    .expect("couldnt find suitable physical device");

            let priorities = [1.0];
            let device_extension_names_raw = [ash::khr::swapchain::NAME.as_ptr()];
            let device_features = vk::PhysicalDeviceFeatures::default();

            // create the logical device
            let device = {
                let queue_families = std::collections::HashSet::from([
                    queue_family_index,
                    transfer_queue_family_index,
                ])
                .into_iter()
                .map(|queue_index| {
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(queue_index as u32)
                        .queue_priorities(&priorities)
                })
                .collect::<Vec<_>>();
                let create_info = vk::DeviceCreateInfo::default()
                    .queue_create_infos(&queue_families)
                    .enabled_features(&device_features)
                    .enabled_extension_names(&device_extension_names_raw);

                instance
                    .create_device(physical_device, &create_info, None)
                    .expect("failed to create device")
            };

            let graphics_queue = device.get_device_queue(queue_family_index as u32, 0);
            let transfer_queue = device.get_device_queue(transfer_queue_family_index as u32, 0);

            let swapchain_device = ash::khr::swapchain::Device::new(&instance, &device);

            let surface_resolution;
            // NOTE: swapchain creation
            let (swapchain, present_images, present_image_views) = {
                let surface_format = surface_instance
                    .get_physical_device_surface_formats(physical_device, surface)
                    .unwrap()[0];
                let surface_capabilities = surface_instance
                    .get_physical_device_surface_capabilities(physical_device, surface)
                    .unwrap();
                let present_modes = surface_instance
                    .get_physical_device_surface_present_modes(physical_device, surface)
                    .unwrap();

                let desired_image_count = u32::min(
                    surface_capabilities.min_image_count + 1,
                    surface_capabilities.max_image_count,
                );
                surface_resolution = match surface_capabilities.current_extent.width {
                    u32::MAX => vk::Extent2D {
                        width: window_size.0,
                        height: window_size.1,
                    },
                    _ => surface_capabilities.current_extent,
                };
                let pre_transform = if surface_capabilities
                    .supported_transforms
                    .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
                {
                    vk::SurfaceTransformFlagsKHR::IDENTITY
                } else {
                    surface_capabilities.current_transform
                };
                let present_mode = present_modes
                    .iter()
                    .copied()
                    .find(|mode| *mode == vk::PresentModeKHR::MAILBOX)
                    .unwrap_or(vk::PresentModeKHR::FIFO);

                let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
                    .surface(surface)
                    .min_image_count(desired_image_count)
                    .image_color_space(surface_format.color_space)
                    .image_format(surface_format.format)
                    .image_extent(surface_resolution)
                    .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                    .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .pre_transform(pre_transform)
                    .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                    .present_mode(present_mode)
                    .clipped(true)
                    .image_array_layers(1);

                let swapchain = swapchain_device
                    .create_swapchain(&swapchain_create_info, None)
                    .expect("failed to create swapchain");

                let present_images = swapchain_device
                    .get_swapchain_images(swapchain)
                    .expect("failed to get swapchain images");
                let present_image_views: Vec<_> = present_images
                    .iter()
                    .map(|&image| {
                        let create_view_info = vk::ImageViewCreateInfo::default()
                            .view_type(vk::ImageViewType::TYPE_2D)
                            .format(surface_format.format)
                            .components(vk::ComponentMapping {
                                r: vk::ComponentSwizzle::R,
                                g: vk::ComponentSwizzle::G,
                                b: vk::ComponentSwizzle::B,
                                a: vk::ComponentSwizzle::A,
                            })
                            .subresource_range(vk::ImageSubresourceRange {
                                aspect_mask: vk::ImageAspectFlags::COLOR,
                                base_mip_level: 0,
                                level_count: 1,
                                base_array_layer: 0,
                                layer_count: 1,
                            })
                            .image(image);
                        device.create_image_view(&create_view_info, None).unwrap()
                    })
                    .collect();

                (swapchain, present_images, present_image_views)
            };

            let (command_pool, transfer_pool) = {
                let pool_create_info = vk::CommandPoolCreateInfo::default()
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                    .queue_family_index(queue_family_index as u32);

                let command_pool = device
                    .create_command_pool(&pool_create_info, None)
                    .expect("failed to create command pool");
                let transfer_pool = device
                    .create_command_pool(
                        &pool_create_info.queue_family_index(transfer_queue_family_index as u32),
                        None,
                    )
                    .expect("failed to create command pool");

                (command_pool, transfer_pool)
            };

            let device_memory_properties =
                instance.get_physical_device_memory_properties(physical_device);

            let (depth_image, depth_image_memory, depth_image_view) = {
                let depth_image_create_info = vk::ImageCreateInfo::default()
                    .image_type(vk::ImageType::TYPE_2D)
                    .format(vk::Format::D16_UNORM)
                    .extent(surface_resolution.into())
                    .mip_levels(1)
                    .array_layers(1)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .tiling(vk::ImageTiling::OPTIMAL)
                    .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE);

                let depth_image = device.create_image(&depth_image_create_info, None).unwrap();

                let depth_image_memory_reqs = device.get_image_memory_requirements(depth_image);
                let depth_image_memory_index = find_memorytype_index(
                    &depth_image_memory_reqs,
                    &device_memory_properties,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                )
                .expect("unable to find suitable memory index for depth image");

                let depth_image_allocate_info = vk::MemoryAllocateInfo::default()
                    .allocation_size(depth_image_memory_reqs.size)
                    .memory_type_index(depth_image_memory_index);

                let depth_image_memory = device
                    .allocate_memory(&depth_image_allocate_info, None)
                    .unwrap();

                device
                    .bind_image_memory(depth_image, depth_image_memory, 0)
                    .expect("failed to bind memory to depth image");

                let depth_image_view_create_info = vk::ImageViewCreateInfo::default()
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::DEPTH)
                            .level_count(1)
                            .layer_count(1),
                    )
                    .image(depth_image)
                    .format(depth_image_create_info.format)
                    .view_type(vk::ImageViewType::TYPE_2D);

                let depth_image_view = device
                    .create_image_view(&depth_image_view_create_info, None)
                    .unwrap();

                (depth_image, depth_image_memory, depth_image_view)
            };

            // TODO: maybe more initialization needed
            CoreRenderData {
                instance,
                device,
                physical_device,

                surface_instance,
                surface,
                queues: Queues {
                    graphics: graphics_queue,
                    transfer: transfer_queue,
                },

                swapchain_device,
                swapchain,
                present_images: present_images
                    .into_iter()
                    .zip(present_image_views.into_iter())
                    .map(|(image, image_view)| VulkanImage(image, image_view))
                    .collect(),

                command_pool,
                transfer_pool,

                depth_image: VulkanObject(
                    VulkanImage(depth_image, depth_image_view),
                    depth_image_memory,
                ),

                debug_callback,
                debug_utils_instance,
            }
        }
    }
}

pub fn find_physical_device(
    instance: &Instance,
    surface_instance: &ash::khr::surface::Instance,
    surface: &vk::SurfaceKHR,
    pdevices: &[vk::PhysicalDevice],
) -> Option<(vk::PhysicalDevice, u32, u32)> {
    unsafe {
        pdevices.iter().find_map(|pdevice| {
            let properties = instance.get_physical_device_queue_family_properties(*pdevice);

            let graphics_queue = match instance
                .get_physical_device_queue_family_properties(*pdevice)
                .iter()
                .enumerate()
                .find_map(|(index, info)| {
                    let supports_graphic_and_surface = info
                        .queue_flags
                        .contains(vk::QueueFlags::GRAPHICS)
                        && surface_instance
                            .get_physical_device_surface_support(*pdevice, index as u32, *surface)
                            .unwrap();
                    if supports_graphic_and_surface {
                        Some(index)
                    } else {
                        None
                    }
                }) {
                Some(v) => v,
                None => return None,
            };

            match find_most_suitable_queue_family(&properties, vk::QueueFlags::TRANSFER, &[]) {
                Some(transfer) => Some((*pdevice, graphics_queue as u32, transfer as u32)),
                None => None,
            }
        })
    }
}

/// finds the queue family with the least flags
fn find_most_suitable_queue_family(
    queue_family_properties: &[vk::QueueFamilyProperties],
    flags: vk::QueueFlags,
    not_queue_index: &[usize],
) -> Option<usize> {
    queue_family_properties
        .iter()
        .enumerate()
        .fold((u32::BITS, None), |(bit_count, index), (i, info)| {
            if not_queue_index.contains(&i) {
                (bit_count, index)
            } else if info.queue_flags.contains(flags)
                && info.queue_flags.as_raw().count_ones() < bit_count
            {
                (info.queue_flags.as_raw().count_ones(), Some(i))
            } else {
                (bit_count, index)
            }
        })
        .1
}
pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        std::borrow::Cow::from("")
    } else {
        std::ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::from("")
    } else {
        std::ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );

    vk::FALSE
}

impl Drop for CoreRenderData {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            self.device.destroy_image_view(self.depth_image.0 .1, None);
            self.device.destroy_image(self.depth_image.0 .0, None);
            self.device.free_memory(self.depth_image.1, None);

            self.present_images.iter().for_each(|image| {
                self.device.destroy_image_view(image.1, None);
                self.device.destroy_image(image.0, None);
            });
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);

            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_command_pool(self.transfer_pool, None);

            self.surface_instance.destroy_surface(self.surface, None);

            self.device.destroy_device(None);

            self.debug_utils_instance
                .destroy_debug_utils_messenger(self.debug_callback, None);

            self.instance.destroy_instance(None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn most_suitable_queue_family_test() {
        let queue_families = vec![
            vk::QueueFamilyProperties::default().queue_flags(
                vk::QueueFlags::TRANSFER | vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE,
            ),
            vk::QueueFamilyProperties::default()
                .queue_flags(vk::QueueFlags::TRANSFER | vk::QueueFlags::GRAPHICS),
            vk::QueueFamilyProperties::default()
                .queue_flags(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE),
            vk::QueueFamilyProperties::default().queue_flags(vk::QueueFlags::TRANSFER),
            vk::QueueFamilyProperties::default()
                .queue_flags(vk::QueueFlags::TRANSFER | vk::QueueFlags::COMPUTE),
        ];

        assert_eq!(
            find_most_suitable_queue_family(&queue_families, vk::QueueFlags::TRANSFER, &[]),
            Some(3)
        );
        assert_eq!(
            find_most_suitable_queue_family(
                &queue_families,
                vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE,
                &[]
            ),
            Some(2)
        );
        assert_eq!(
            find_most_suitable_queue_family(&queue_families, vk::QueueFlags::TRANSFER, &[3]),
            Some(1)
        );
    }
}
