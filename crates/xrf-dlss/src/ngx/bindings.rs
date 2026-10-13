#![allow(
  non_upper_case_globals,
  non_camel_case_types,
  non_snake_case,
  unused,
  unsafe_op_in_unsafe_fn,
  clippy::all
)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// The Vulkan types NGX's signatures name, which bindgen leaves to ash.
type PFN_vkGetDeviceProcAddr = ash::vk::PFN_vkGetDeviceProcAddr;
type PFN_vkGetInstanceProcAddr = ash::vk::PFN_vkGetInstanceProcAddr;
type VkBuffer = ash::vk::Buffer;
type VkCommandBuffer = ash::vk::CommandBuffer;
type VkDevice = ash::vk::Device;
type VkExtensionProperties = ash::vk::ExtensionProperties;
type VkFormat = ash::vk::Format;
type VkImage = ash::vk::Image;
type VkImageSubresourceRange = ash::vk::ImageSubresourceRange;
type VkImageView = ash::vk::ImageView;
type VkInstance = ash::vk::Instance;
type VkPhysicalDevice = ash::vk::PhysicalDevice;
